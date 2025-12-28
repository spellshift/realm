use serde::{Deserialize, Serialize};

/// Individual callback configuration
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct CallbackConfig {
    /// URI for this callback (must specify a scheme, e.g. `http://` or `dns://`)
    pub uri: String,

    /// Duration between callbacks for this URI, in seconds
    /// Default: `5`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,

    /// Duration to wait before retrying this callback if an error occurs, in seconds
    /// Default: `5`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_interval: Option<u32>,

    /// Override system settings for proxy URI over HTTP(S)
    /// Only supported for http1 and grpc transports
    /// (must specify a scheme, e.g. `https://`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_uri: Option<String>,

    /// DNS-over-HTTPS provider for gRPC transport
    /// Valid values: "cloudflare", "google", "quad9"
    /// If not specified, system DNS resolver will be used
    /// Only supported for grpc transport
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doh_provider: Option<String>,
}

/// Build configuration structure matching the environment variables
/// documented in the user guide
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct ImixBuildConfig {
    /// List of callback configurations
    /// Each callback can have its own URI, interval, retry_interval, and proxy_uri
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callbacks: Option<Vec<CallbackConfig>>,

    /// The public key for the tavern server
    /// (obtain from server using `curl $IMIX_CALLBACK_URI/status`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_pubkey: Option<String>,

    /// Manually specify the host ID for this beacon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_id: Option<String>,

    /// Imix will only do one callback and execution of queued tasks
    /// Default: false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_once: Option<bool>,

    /// Feature flags for conditional compilation
    /// Valid values: grpc, http1, dns, win_service
    /// Note: DoH is now configured per-callback via doh_provider field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<String>>,
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    EmptyCallbacks,
    UnsupportedProxyTransport {
        uri: String,
        proxy_uri: String,
    },
    UnsupportedDohTransport {
        uri: String,
        doh_provider: String,
    },
    InvalidDohProvider {
        provider: String,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyCallbacks => {
                write!(
                    f,
                    "At least one callback must be configured. The callbacks list is empty."
                )
            }
            ValidationError::UnsupportedProxyTransport { uri, proxy_uri } => {
                write!(
                    f,
                    "proxy_uri is only supported for HTTP/HTTPS (gRPC) and HTTP1 transports. \
                    Callback URI: {}, proxy_uri: {}. \
                    Supported URI schemes: http://, https://, http1://, https1://",
                    uri, proxy_uri
                )
            }
            ValidationError::UnsupportedDohTransport { uri, doh_provider } => {
                write!(
                    f,
                    "doh_provider is only supported for gRPC callbacks (http:// or https://). \
                    Callback URI: {}, doh_provider: {}.",
                    uri, doh_provider
                )
            }
            ValidationError::InvalidDohProvider { provider } => {
                write!(
                    f,
                    "Invalid doh_provider '{}'. Valid values are: cloudflare, google, quad9",
                    provider
                )
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Parse YAML configuration from string content
pub fn parse_yaml_build_config(
    yaml_content: &str,
) -> Result<ImixBuildConfig, Box<dyn std::error::Error>> {
    let config: ImixBuildConfig = serde_yaml::from_str(yaml_content)?;
    Ok(config)
}

/// Validate the build configuration
pub fn validate_config(config: &ImixBuildConfig) -> Result<(), ValidationError> {
    if let Some(ref callbacks) = config.callbacks {
        // Validate at least one callback is configured
        if callbacks.is_empty() {
            return Err(ValidationError::EmptyCallbacks);
        }

        // Validate proxy_uri and doh_provider are only set for supported transports
        for callback in callbacks {
            let uri_lower = callback.uri.to_lowercase();

            // Validate proxy_uri is only used with supported transports (http1 and grpc)
            if let Some(ref proxy_uri) = callback.proxy_uri {
                let is_supported = uri_lower.starts_with("http://")
                    || uri_lower.starts_with("https://")
                    || uri_lower.starts_with("http1://")
                    || uri_lower.starts_with("https1://");

                if !is_supported {
                    return Err(ValidationError::UnsupportedProxyTransport {
                        uri: callback.uri.clone(),
                        proxy_uri: proxy_uri.clone(),
                    });
                }
            }

            // Validate doh_provider
            if let Some(ref doh_provider) = callback.doh_provider {
                if !uri_lower.starts_with("https://") && !uri_lower.starts_with("http://") {
                    return Err(ValidationError::UnsupportedDohTransport {
                        uri: callback.uri.clone(),
                        doh_provider: doh_provider.clone(),
                    });
                }

                // Validate doh_provider value
                let provider_lower = doh_provider.to_lowercase();
                if provider_lower != "cloudflare"
                    && provider_lower != "google"
                    && provider_lower != "quad9"
                {
                    return Err(ValidationError::InvalidDohProvider {
                        provider: doh_provider.clone(),
                    });
                }
            }
        }
    }

    Ok(())
}

/// Prepare callbacks for runtime by encoding doh_provider in URI query params
pub fn prepare_callbacks(callbacks: &[CallbackConfig]) -> Vec<CallbackConfig> {
    callbacks
        .iter()
        .map(|cb| {
            let mut runtime_cb = cb.clone();

            // If doh_provider is set, append it to the URI as a query parameter
            if let Some(ref doh_provider) = cb.doh_provider {
                let separator = if runtime_cb.uri.contains('?') {
                    "&"
                } else {
                    "?"
                };
                runtime_cb.uri = format!("{}{}doh={}", runtime_cb.uri, separator, doh_provider);
                // Clear doh_provider from the struct since it's now in the URI
                runtime_cb.doh_provider = None;
            }

            runtime_cb
        })
        .collect()
}

/// Find the first HTTPS callback in the list
pub fn find_first_https_callback(callbacks: &[CallbackConfig]) -> Option<&CallbackConfig> {
    callbacks
        .iter()
        .find(|cb| cb.uri.to_lowercase().starts_with("https://"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_config() {
        let yaml = r#"
callbacks:
  - uri: "https://example.com:8000"
    interval: 10
    retry_interval: 5
server_pubkey: "test-key"
"#;

        let config = parse_yaml_build_config(yaml).unwrap();
        assert!(config.callbacks.is_some());
        let callbacks = config.callbacks.unwrap();
        assert_eq!(callbacks.len(), 1);
        assert_eq!(callbacks[0].uri, "https://example.com:8000");
        assert_eq!(callbacks[0].interval, Some(10));
        assert_eq!(callbacks[0].retry_interval, Some(5));
        assert_eq!(config.server_pubkey, Some("test-key".to_string()));
    }

    #[test]
    fn test_parse_multiple_callbacks() {
        let yaml = r#"
callbacks:
  - uri: "dns://8.8.8.8:53?domain=test.com"
    interval: 20
  - uri: "https://backup.com:443"
    interval: 60
    proxy_uri: "http://proxy:8080"
"#;

        let config = parse_yaml_build_config(yaml).unwrap();
        let callbacks = config.callbacks.unwrap();
        assert_eq!(callbacks.len(), 2);
        assert_eq!(callbacks[0].uri, "dns://8.8.8.8:53?domain=test.com");
        assert_eq!(callbacks[1].proxy_uri, Some("http://proxy:8080".to_string()));
    }

    #[test]
    fn test_validate_empty_callbacks() {
        let config = ImixBuildConfig {
            callbacks: Some(vec![]),
            server_pubkey: None,
            host_id: None,
            run_once: None,
            features: None,
        };

        let result = validate_config(&config);
        assert!(matches!(result, Err(ValidationError::EmptyCallbacks)));
    }

    #[test]
    fn test_validate_proxy_uri_with_dns() {
        let config = ImixBuildConfig {
            callbacks: Some(vec![CallbackConfig {
                uri: "dns://8.8.8.8:53?domain=test.com".to_string(),
                interval: None,
                retry_interval: None,
                proxy_uri: Some("http://proxy:8080".to_string()),
                doh_provider: None,
            }]),
            server_pubkey: None,
            host_id: None,
            run_once: None,
            features: None,
        };

        let result = validate_config(&config);
        assert!(matches!(
            result,
            Err(ValidationError::UnsupportedProxyTransport { .. })
        ));
    }

    #[test]
    fn test_validate_proxy_uri_with_https() {
        let config = ImixBuildConfig {
            callbacks: Some(vec![CallbackConfig {
                uri: "https://example.com:443".to_string(),
                interval: None,
                retry_interval: None,
                proxy_uri: Some("http://proxy:8080".to_string()),
                doh_provider: None,
            }]),
            server_pubkey: None,
            host_id: None,
            run_once: None,
            features: None,
        };

        let result = validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_doh_provider_with_dns() {
        let config = ImixBuildConfig {
            callbacks: Some(vec![CallbackConfig {
                uri: "dns://8.8.8.8:53?domain=test.com".to_string(),
                interval: None,
                retry_interval: None,
                proxy_uri: None,
                doh_provider: Some("cloudflare".to_string()),
            }]),
            server_pubkey: None,
            host_id: None,
            run_once: None,
            features: None,
        };

        let result = validate_config(&config);
        assert!(matches!(
            result,
            Err(ValidationError::UnsupportedDohTransport { .. })
        ));
    }

    #[test]
    fn test_validate_invalid_doh_provider() {
        let config = ImixBuildConfig {
            callbacks: Some(vec![CallbackConfig {
                uri: "https://example.com:443".to_string(),
                interval: None,
                retry_interval: None,
                proxy_uri: None,
                doh_provider: Some("invalid".to_string()),
            }]),
            server_pubkey: None,
            host_id: None,
            run_once: None,
            features: None,
        };

        let result = validate_config(&config);
        assert!(matches!(
            result,
            Err(ValidationError::InvalidDohProvider { .. })
        ));
    }

    #[test]
    fn test_validate_valid_doh_providers() {
        for provider in &["cloudflare", "google", "quad9", "Cloudflare", "GOOGLE"] {
            let config = ImixBuildConfig {
                callbacks: Some(vec![CallbackConfig {
                    uri: "https://example.com:443".to_string(),
                    interval: None,
                    retry_interval: None,
                    proxy_uri: None,
                    doh_provider: Some(provider.to_string()),
                }]),
                server_pubkey: None,
                host_id: None,
                run_once: None,
                features: None,
            };

            let result = validate_config(&config);
            assert!(result.is_ok(), "Failed for provider: {}", provider);
        }
    }

    #[test]
    fn test_prepare_callbacks_with_doh() {
        let callbacks = vec![
            CallbackConfig {
                uri: "https://example.com:443".to_string(),
                interval: Some(10),
                retry_interval: Some(5),
                proxy_uri: None,
                doh_provider: Some("cloudflare".to_string()),
            },
            CallbackConfig {
                uri: "https://backup.com:443?foo=bar".to_string(),
                interval: Some(30),
                retry_interval: Some(15),
                proxy_uri: None,
                doh_provider: Some("google".to_string()),
            },
        ];

        let prepared = prepare_callbacks(&callbacks);
        assert_eq!(prepared.len(), 2);
        assert_eq!(prepared[0].uri, "https://example.com:443?doh=cloudflare");
        assert_eq!(prepared[0].doh_provider, None);
        assert_eq!(
            prepared[1].uri,
            "https://backup.com:443?foo=bar&doh=google"
        );
        assert_eq!(prepared[1].doh_provider, None);
    }

    #[test]
    fn test_prepare_callbacks_without_doh() {
        let callbacks = vec![CallbackConfig {
            uri: "https://example.com:443".to_string(),
            interval: Some(10),
            retry_interval: Some(5),
            proxy_uri: None,
            doh_provider: None,
        }];

        let prepared = prepare_callbacks(&callbacks);
        assert_eq!(prepared.len(), 1);
        assert_eq!(prepared[0].uri, "https://example.com:443");
        assert_eq!(prepared[0].doh_provider, None);
    }

    #[test]
    fn test_find_first_https_callback() {
        let callbacks = vec![
            CallbackConfig {
                uri: "dns://8.8.8.8:53?domain=test.com".to_string(),
                interval: Some(10),
                retry_interval: Some(5),
                proxy_uri: None,
                doh_provider: None,
            },
            CallbackConfig {
                uri: "https://example.com:443".to_string(),
                interval: Some(20),
                retry_interval: Some(10),
                proxy_uri: None,
                doh_provider: None,
            },
            CallbackConfig {
                uri: "https://backup.com:443".to_string(),
                interval: Some(30),
                retry_interval: Some(15),
                proxy_uri: None,
                doh_provider: None,
            },
        ];

        let https = find_first_https_callback(&callbacks);
        assert!(https.is_some());
        assert_eq!(https.unwrap().uri, "https://example.com:443");
    }

    #[test]
    fn test_find_first_https_callback_none() {
        let callbacks = vec![CallbackConfig {
            uri: "dns://8.8.8.8:53?domain=test.com".to_string(),
            interval: Some(10),
            retry_interval: Some(5),
            proxy_uri: None,
            doh_provider: None,
        }];

        let https = find_first_https_callback(&callbacks);
        assert!(https.is_none());
    }

    #[test]
    fn test_parse_with_features() {
        let yaml = r#"
callbacks:
  - uri: "https://example.com:8000"
features:
  - grpc
  - http1
  - dns
"#;

        let config = parse_yaml_build_config(yaml).unwrap();
        assert!(config.features.is_some());
        let features = config.features.unwrap();
        assert_eq!(features.len(), 3);
        assert!(features.contains(&"grpc".to_string()));
        assert!(features.contains(&"http1".to_string()));
        assert!(features.contains(&"dns".to_string()));
    }
}
