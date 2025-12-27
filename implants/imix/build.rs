use serde::{Deserialize, Serialize};
use std::env;

/// Individual callback configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
struct CallbackConfig {
    /// URI for this callback (must specify a scheme, e.g. `http://` or `dns://`)
    uri: String,

    /// Duration between callbacks for this URI, in seconds
    /// Default: `5`
    #[serde(skip_serializing_if = "Option::is_none")]
    interval: Option<u32>,

    /// Duration to wait before retrying this callback if an error occurs, in seconds
    /// Default: `5`
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_interval: Option<u32>,
}

/// Build configuration structure matching the environment variables
/// documented in the user guide
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
struct ImixBuildConfig {
    /// List of callback configurations (new format)
    /// Each callback can have its own URI, interval, and retry_interval
    #[serde(skip_serializing_if = "Option::is_none")]
    callbacks: Option<Vec<CallbackConfig>>,

    /// URI for initial callbacks (legacy single callback format)
    /// (must specify a scheme, e.g. `http://` or `dns://`)
    /// Default: `http://127.0.0.1:8000`
    #[serde(skip_serializing_if = "Option::is_none")]
    callback_uri: Option<String>,

    /// The public key for the tavern server
    /// (obtain from server using `curl $IMIX_CALLBACK_URI/status`)
    #[serde(skip_serializing_if = "Option::is_none")]
    server_pubkey: Option<String>,

    /// Duration between callbacks, in seconds (legacy format)
    /// Default: `5`
    #[serde(skip_serializing_if = "Option::is_none")]
    callback_interval: Option<u32>,

    /// Duration to wait before restarting the agent loop if an error occurs, in seconds (legacy format)
    /// Default: `5`
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_interval: Option<u32>,

    /// Override system settings for proxy URI over HTTP(S)
    /// (must specify a scheme, e.g. `https://`)
    #[serde(skip_serializing_if = "Option::is_none")]
    proxy_uri: Option<String>,

    /// Manually specify the host ID for this beacon
    #[serde(skip_serializing_if = "Option::is_none")]
    host_id: Option<String>,

    /// Imix will only do one callback and execution of queued tasks
    /// Default: false
    #[serde(skip_serializing_if = "Option::is_none")]
    run_once: Option<bool>,

    /// Feature flags for conditional compilation
    /// Valid values: grpc, http1, dns, transport-grpc-doh, win_service
    #[serde(skip_serializing_if = "Option::is_none")]
    features: Option<Vec<String>>,
}

/// Parse YAML configuration from string content
fn parse_yaml_build_config(yaml_content: &str) -> Result<ImixBuildConfig, Box<dyn std::error::Error>> {
    let config: ImixBuildConfig = serde_yaml::from_str(yaml_content)?;
    Ok(config)
}

/// Apply build configuration by setting cargo environment variables
fn apply_build_config(config: &ImixBuildConfig) {
    // Handle callbacks - new format takes precedence over legacy format
    if let Some(ref callbacks) = config.callbacks {
        if !callbacks.is_empty() {
            // Serialize callbacks list as JSON for runtime consumption
            match serde_json::to_string(callbacks) {
                Ok(json) => {
                    println!("cargo:rustc-env=IMIX_CALLBACKS={}", json);
                    println!("cargo:warning=Setting IMIX_CALLBACKS with {} callback(s)", callbacks.len());

                    // Also set the first callback as the primary for backward compatibility
                    if let Some(first) = callbacks.first() {
                        println!("cargo:rustc-env=IMIX_CALLBACK_URI={}", first.uri);
                        println!("cargo:warning=Setting IMIX_CALLBACK_URI={} (primary)", first.uri);

                        if let Some(interval) = first.interval {
                            println!("cargo:rustc-env=IMIX_CALLBACK_INTERVAL={}", interval);
                            println!("cargo:warning=Setting IMIX_CALLBACK_INTERVAL={} (primary)", interval);
                        }

                        if let Some(retry) = first.retry_interval {
                            println!("cargo:rustc-env=IMIX_RETRY_INTERVAL={}", retry);
                            println!("cargo:warning=Setting IMIX_RETRY_INTERVAL={} (primary)", retry);
                        }
                    }
                }
                Err(e) => {
                    println!("cargo:warning=Failed to serialize callbacks to JSON: {}", e);
                }
            }
        }
    } else {
        // Legacy single callback format
        if let Some(ref callback_uri) = config.callback_uri {
            println!("cargo:rustc-env=IMIX_CALLBACK_URI={}", callback_uri);
            println!("cargo:warning=Setting IMIX_CALLBACK_URI={}", callback_uri);
        }

        if let Some(callback_interval) = config.callback_interval {
            println!("cargo:rustc-env=IMIX_CALLBACK_INTERVAL={}", callback_interval);
            println!("cargo:warning=Setting IMIX_CALLBACK_INTERVAL={}", callback_interval);
        }

        if let Some(retry_interval) = config.retry_interval {
            println!("cargo:rustc-env=IMIX_RETRY_INTERVAL={}", retry_interval);
            println!("cargo:warning=Setting IMIX_RETRY_INTERVAL={}", retry_interval);
        }
    }

    // Set other configuration options
    if let Some(ref server_pubkey) = config.server_pubkey {
        println!("cargo:rustc-env=IMIX_SERVER_PUBKEY={}", server_pubkey);
        println!("cargo:warning=Setting IMIX_SERVER_PUBKEY=(redacted for security)");
    }

    if let Some(ref proxy_uri) = config.proxy_uri {
        println!("cargo:rustc-env=IMIX_PROXY_URI={}", proxy_uri);
        println!("cargo:warning=Setting IMIX_PROXY_URI={}", proxy_uri);
    }

    if let Some(ref host_id) = config.host_id {
        println!("cargo:rustc-env=IMIX_HOST_ID={}", host_id);
        println!("cargo:warning=Setting IMIX_HOST_ID={}", host_id);
    }

    if let Some(run_once) = config.run_once {
        println!("cargo:rustc-env=IMIX_RUN_ONCE={}", run_once);
        println!("cargo:warning=Setting IMIX_RUN_ONCE={}", run_once);
    }

    // Handle feature flags for conditional compilation
    if let Some(ref features) = config.features {
        println!("cargo:warning=Configured features: {:?}", features);

        // Set cfg flags based on features
        for feature in features {
            match feature.as_str() {
                "grpc" => println!("cargo:rustc-cfg=feature=\"transport_grpc\""),
                "http1" => println!("cargo:rustc-cfg=feature=\"transport_http1\""),
                "dns" => println!("cargo:rustc-cfg=feature=\"transport_dns\""),
                "transport-grpc-doh" => println!("cargo:rustc-cfg=feature=\"transport_grpc_doh\""),
                "win_service" => println!("cargo:rustc-cfg=feature=\"win_service\""),
                _ => println!("cargo:warning=Unknown feature: {}", feature),
            }
        }
    }
}

fn main() {
    // Try to read YAML configuration from environment variable
    if let Ok(yaml_content) = env::var("IMIX_BUILD_CONFIG") {
        println!("cargo:warning=Found IMIX_BUILD_CONFIG environment variable");
        println!("cargo:rerun-if-env-changed=IMIX_BUILD_CONFIG");

        match parse_yaml_build_config(&yaml_content) {
            Ok(config) => {
                println!("cargo:warning=Successfully parsed YAML build configuration");
                apply_build_config(&config);
            }
            Err(e) => {
                println!("cargo:warning=Failed to parse YAML build configuration: {}", e);
                println!("cargo:warning=Continuing with default/environment variable configuration");
            }
        }
    } else {
        println!("cargo:warning=No IMIX_BUILD_CONFIG environment variable found");
        println!("cargo:warning=Set IMIX_BUILD_CONFIG with YAML content to use YAML-based configuration");
        println!("cargo:warning=Using individual environment variables for build configuration");
    }

    // Windows-specific build configuration
    #[cfg(target_os = "windows")]
    static_vcruntime::metabuild();
}
