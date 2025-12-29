use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Transport configuration parsed from URI query parameters
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Full URI including query parameters (required)
    pub uri: String,
    /// Retry interval in seconds (universal parameter with default fallback)
    pub retry_interval: u64,
    /// Callback interval in seconds (universal parameter with default fallback)
    pub callback_interval: u64,
    /// Transport-specific parameters (e.g., proxy_uri for GRPC, doh for GRPC, domain/type for DNS)
    pub transport_specific: HashMap<String, String>,
}

/// Parse a transport URI into base URI and configuration
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
pub fn parse_transport_uri(uri: &str) -> Result<(String, TransportConfig)> {
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

    Ok((
        base_uri,
        TransportConfig {
            uri: uri.to_string(),
            retry_interval,
            callback_interval,
            transport_specific,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_complete_grpc_uri() {
        let uri = "grpc://example.com:443?retry_interval=10&callback_interval=20&proxy_uri=http://proxy:8080&doh=cloudflare";
        let (base, config) = parse_transport_uri(uri).unwrap();
        assert_eq!(base, "grpc://example.com:443");
        assert_eq!(config.uri, uri);
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
        let (base, config) = parse_transport_uri(uri).unwrap();
        assert_eq!(base, "http1://example.com");
        assert_eq!(config.uri, uri);
        assert_eq!(config.retry_interval, 5);
        assert_eq!(config.callback_interval, 5);
        assert!(config.transport_specific.is_empty());
    }

    #[test]
    fn test_parse_dns_uri() {
        let uri = "dns://8.8.8.8?retry_interval=5&callback_interval=10&domain=dnsc2.realm&type=txt";
        let (base, config) = parse_transport_uri(uri).unwrap();
        assert_eq!(base, "dns://8.8.8.8");
        assert_eq!(config.uri, uri);
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
