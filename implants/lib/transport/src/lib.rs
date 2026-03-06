use anyhow::{anyhow, Result};
use pb::c2::transport::Type as TransportType;

#[cfg(any(feature = "grpc", feature = "http1"))]
mod tls_utils;

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

pub fn create_transport(transport: &pb::c2::Transport) -> Result<Box<dyn Transport + Send + Sync>> {
    let transport_type = transport.r#type;

    // Match on the transport type enum
    match TransportType::try_from(transport_type) {
        Ok(TransportType::TransportGrpc) => {
            #[cfg(feature = "grpc")]
            return Ok(Box::new(grpc::GRPC::new(transport)?));
            #[cfg(not(feature = "grpc"))]
            return Err(anyhow!("gRPC transport not enabled"));
        }
        Ok(TransportType::TransportHttp1) => {
            #[cfg(feature = "http1")]
            return Ok(Box::new(http::HTTP::new(transport)?));
            #[cfg(not(feature = "http1"))]
            return Err(anyhow!("http1 transport not enabled"));
        }
        Ok(TransportType::TransportDns) => {
            #[cfg(feature = "dns")]
            return Ok(Box::new(dns::DNS::new(transport)?));
            #[cfg(not(feature = "dns"))]
            return Err(anyhow!("DNS transport not enabled"));
        }
        Ok(TransportType::TransportUnspecified) | Err(_) => {
            Err(anyhow!("Invalid or unspecified transport type"))
        }
    }
}

pub fn empty_transport() -> Box<dyn Transport + Send + Sync> {
    let transport = pb::c2::Transport {
        uri: "http://127.0.0.1".to_string(),
        r#type: TransportType::TransportHttp1 as i32,
        ..Default::default()
    };
    create_transport(&transport).expect("Failed to create empty transport")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a test transport with a specific URI, transport type, and extra params
    fn create_test_transport(uri: &str, transport_type: i32, extra: &str) -> pb::c2::Transport {
        pb::c2::Transport {
            uri: uri.to_string(),
            interval: 5,
            r#type: transport_type,
            extra: extra.to_string(),
            jitter: 0.0,
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
            let transport = create_test_transport(uri, TransportType::TransportGrpc as i32, "{}");
            let result = create_transport(&transport);

            // 1. Assert strictly on the Variant type
            assert!(result.is_ok(), "URI '{}' did not resolve to Grpc", uri);
            assert_eq!(result.unwrap().name(), "grpc");
        }
    }

    #[tokio::test]
    #[cfg(feature = "http1")]
    async fn test_routes_to_http1_transport() {
        // All these prefixes should result in the Http1 variant
        let inputs = vec!["http1://127.0.0.1:8080", "https1://127.0.0.1:8080"];

        for uri in inputs {
            let transport = create_test_transport(uri, TransportType::TransportHttp1 as i32, "{}");
            let result = create_transport(&transport);

            assert!(result.is_ok(), "URI '{}' did not resolve to Http", uri);
            assert_eq!(result.unwrap().name(), "http");
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
            let transport = create_test_transport(uri, TransportType::TransportDns as i32, extra);
            let result = create_transport(&transport);

            assert!(
                result.is_ok(),
                "URI '{}' with extra '{}' did not resolve to Dns",
                uri,
                extra
            );
            assert_eq!(result.unwrap().name(), "dns");
        }
    }

    #[tokio::test]
    #[cfg(not(feature = "grpc"))]
    async fn test_grpc_disabled_error() {
        // If the feature is off, these should error out
        let inputs = vec!["grpc://foo", "grpcs://foo", "http://foo"];
        for uri in inputs {
            let transport = create_test_transport(uri, TransportType::TransportGrpc as i32, "{}");
            let result = create_transport(&transport);
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
        let transport = create_test_transport(
            "ftp://example.com",
            TransportType::TransportUnspecified as i32,
            "{}",
        );
        let result = create_transport(&transport);
        assert!(result.is_err(), "Expected error for unknown transport type");
    }
}
