use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Transport configuration parsed from URI query parameters
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Base URI without query parameters (required)
    pub base_uri: String,
    /// Transport type: "grpc", "http1", or "dns"
    pub transport_type: String,
    /// Retry interval in seconds (universal parameter with default fallback)
    pub retry_interval: u64,
    /// Callback interval in seconds (universal parameter with default fallback)
    pub callback_interval: u64,
    /// Transport-specific parameters (e.g., proxy_uri for GRPC, doh for GRPC, domain/type for DNS)
    pub transport_specific: HashMap<String, String>,
}

/// Parse a transport URI into configuration
///
/// # Example URIs
/// - `grpc://c2.example.com:443?retry_interval=5&callback_interval=10&proxy_uri=http://proxy:8080&doh=cloudflare`
/// - `http1://c2.example.com?retry_interval=5&callback_interval=10`
/// - `dns://8.8.8.8?retry_interval=5&callback_interval=10&domain=dnsc2.realm&type=txt`
///
/// # Parameters
/// - `retry_interval`: Universal parameter (defaults to 5)
/// - `callback_interval`: Universal parameter (defaults to 5)
/// - All other parameters go into `transport_specific` for transport-specific handling
pub fn parse_transport_uri(uri: &str) -> Result<TransportConfig> {
    let parsed = url::Url::parse(uri)?;

    // Extract base URI (scheme + host + port + path, no query)
    let base_uri = if let Some(port) = parsed.port() {
        format!(
            "{}://{}:{}{}",
            parsed.scheme(),
            parsed.host_str().unwrap_or(""),
            port,
            parsed.path()
        )
    } else {
        format!(
            "{}://{}{}",
            parsed.scheme(),
            parsed.host_str().unwrap_or(""),
            parsed.path()
        )
    };

    // Determine transport type from scheme
    let transport_type = match parsed.scheme() {
        "http" | "https" | "grpc" | "grpcs" => "grpc",
        "http1" | "https1" => "http1",
        "dns" => "dns",
        _ => return Err(anyhow!("Unknown transport scheme: {}", parsed.scheme())),
    };

    // Parse query parameters
    let mut retry_interval = None;
    let mut callback_interval = None;
    let mut transport_specific = HashMap::new();

    for (key, value) in parsed.query_pairs() {
        match key.as_ref() {
            "retry_interval" => {
                retry_interval = Some(
                    value
                        .parse::<u64>()
                        .map_err(|_| anyhow!("Invalid retry_interval: {}", value))?,
                );
            }
            "callback_interval" => {
                callback_interval = Some(
                    value
                        .parse::<u64>()
                        .map_err(|_| anyhow!("Invalid callback_interval: {}", value))?,
                );
            }
            // All other params go into transport_specific
            _ => {
                transport_specific.insert(key.to_string(), value.to_string());
            }
        }
    }

    // Fallback to defaults if not provided
    let retry_interval = retry_interval.unwrap_or(5);
    let callback_interval = callback_interval.unwrap_or(5);

    Ok(TransportConfig {
        base_uri,
        transport_type: transport_type.to_string(),
        retry_interval,
        callback_interval,
        transport_specific,
    })
}

/// Convert ActiveCallback protobuf to TransportConfig
///
/// This function extracts configuration from the structured ActiveCallback message
/// and converts it to the TransportConfig format used by transport implementations.
///
/// # Parameters
/// - `retry_interval`: Directly from ActiveCallback.retry_interval
/// - `callback_interval`: Directly from ActiveCallback.callback_interval
/// - `callback_uri`: Base URI from ActiveCallback.callback_uri
/// - `transport_config`: Parsed JSON from ActiveCallback.transport_config
pub fn active_callback_to_transport_config(
    active_callback: &pb::config::ActiveCallback,
) -> Result<TransportConfig> {
    // Parse base URI to determine transport type
    let parsed = url::Url::parse(&active_callback.callback_uri)?;

    // Determine transport type from scheme
    let transport_type = match parsed.scheme() {
        "http" | "https" | "grpc" | "grpcs" => "grpc",
        "http1" | "https1" => "http1",
        "dns" => "dns",
        _ => return Err(anyhow!("Unknown transport scheme: {}", parsed.scheme())),
    };

    // Parse transport_config JSON to HashMap
    let transport_specific: HashMap<String, String> =
        serde_json::from_str(&active_callback.transport_config).unwrap_or_else(|_| HashMap::new());

    Ok(TransportConfig {
        base_uri: active_callback.callback_uri.clone(),
        transport_type: transport_type.to_string(),
        retry_interval: active_callback.retry_interval,
        callback_interval: active_callback.callback_interval,
        transport_specific,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_complete_grpc_uri() {
        let uri = "grpc://example.com:443?retry_interval=10&callback_interval=20&proxy_uri=http://proxy:8080&doh=cloudflare";
        let config = parse_transport_uri(uri).unwrap();
        assert_eq!(config.base_uri, "grpc://example.com:443");
        assert_eq!(config.transport_type, "grpc");
        assert_eq!(config.retry_interval, 10);
        assert_eq!(config.callback_interval, 20);
        assert_eq!(
            config.transport_specific.get("proxy_uri"),
            Some(&"http://proxy:8080".to_string())
        );
        assert_eq!(
            config.transport_specific.get("doh"),
            Some(&"cloudflare".to_string())
        );
    }

    #[test]
    fn test_parse_with_defaults() {
        let uri = "http1://example.com";
        let config = parse_transport_uri(uri).unwrap();
        assert_eq!(config.base_uri, "http1://example.com");
        assert_eq!(config.transport_type, "http1");
        assert_eq!(config.retry_interval, 5);
        assert_eq!(config.callback_interval, 5);
        assert!(config.transport_specific.is_empty());
    }

    #[test]
    fn test_parse_dns_uri() {
        let uri = "dns://8.8.8.8?retry_interval=5&callback_interval=10&domain=dnsc2.realm&type=txt";
        let config = parse_transport_uri(uri).unwrap();
        assert_eq!(config.base_uri, "dns://8.8.8.8");
        assert_eq!(config.transport_type, "dns");
        assert_eq!(config.retry_interval, 5);
        assert_eq!(config.callback_interval, 10);
        assert_eq!(
            config.transport_specific.get("domain"),
            Some(&"dnsc2.realm".to_string())
        );
        assert_eq!(
            config.transport_specific.get("type"),
            Some(&"txt".to_string())
        );
    }

    #[test]
    fn test_parse_invalid_retry_interval() {
        let uri = "grpc://example.com?retry_interval=invalid&callback_interval=10";
        let result = parse_transport_uri(uri);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid retry_interval"));
    }
}
