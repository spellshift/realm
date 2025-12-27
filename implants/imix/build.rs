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

    /// Override system settings for proxy URI over HTTP(S)
    /// Only supported for http1 and grpc transports
    /// (must specify a scheme, e.g. `https://`)
    #[serde(skip_serializing_if = "Option::is_none")]
    proxy_uri: Option<String>,

    /// DNS-over-HTTPS provider for gRPC transport
    /// Valid values: "cloudflare", "google", "quad9"
    /// If not specified, system DNS resolver will be used
    /// Only supported for grpc transport
    #[serde(skip_serializing_if = "Option::is_none")]
    doh_provider: Option<String>,
}

/// Build configuration structure matching the environment variables
/// documented in the user guide
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
struct ImixBuildConfig {
    /// List of callback configurations
    /// Each callback can have its own URI, interval, retry_interval, and proxy_uri
    #[serde(skip_serializing_if = "Option::is_none")]
    callbacks: Option<Vec<CallbackConfig>>,

    /// The public key for the tavern server
    /// (obtain from server using `curl $IMIX_CALLBACK_URI/status`)
    #[serde(skip_serializing_if = "Option::is_none")]
    server_pubkey: Option<String>,

    /// Manually specify the host ID for this beacon
    #[serde(skip_serializing_if = "Option::is_none")]
    host_id: Option<String>,

    /// Imix will only do one callback and execution of queued tasks
    /// Default: false
    #[serde(skip_serializing_if = "Option::is_none")]
    run_once: Option<bool>,

    /// Feature flags for conditional compilation
    /// Valid values: grpc, http1, dns, win_service
    /// Note: DoH is now configured per-callback via doh_provider field
    #[serde(skip_serializing_if = "Option::is_none")]
    features: Option<Vec<String>>,
}

/// Parse YAML configuration from string content
fn parse_yaml_build_config(
    yaml_content: &str,
) -> Result<ImixBuildConfig, Box<dyn std::error::Error>> {
    let config: ImixBuildConfig = serde_yaml::from_str(yaml_content)?;
    Ok(config)
}

/// Apply build configuration by setting cargo environment variables
fn apply_build_config(config: &ImixBuildConfig) {
    // Handle callbacks
    if let Some(ref callbacks) = config.callbacks {
        // Validate at least one callback is configured
        if callbacks.is_empty() {
            panic!(
                "cargo:error=At least one callback must be configured. \
                The callbacks list is empty."
            );
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
                    panic!(
                        "cargo:error=proxy_uri is only supported for HTTP/HTTPS (gRPC) and HTTP1 transports. \
                        Callback URI: {}, proxy_uri: {}. \
                        Supported URI schemes: http://, https://, http1://, https1://",
                        callback.uri, proxy_uri
                    );
                }
            }

            // Validate doh_provider
            if let Some(ref doh_provider) = callback.doh_provider {
                if !uri_lower.starts_with("https://") && !uri_lower.starts_with("http://") {
                    panic!(
                        "cargo:error=doh_provider is only supported for gRPC callbacks (http:// or https://). \
                        Callback URI: {}, doh_provider: {}.",
                        callback.uri, doh_provider
                    );
                }

                // Validate doh_provider value
                let provider_lower = doh_provider.to_lowercase();
                if provider_lower != "cloudflare"
                    && provider_lower != "google"
                    && provider_lower != "quad9"
                {
                    panic!(
                        "cargo:error=Invalid doh_provider '{}'. \
                        Valid values are: cloudflare, google, quad9",
                        doh_provider
                    );
                }
            }
        }

        // Prepare callbacks for runtime by encoding doh_provider in URI query params
        let runtime_callbacks: Vec<CallbackConfig> = callbacks
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
            .collect();

        // Serialize callbacks list as YAML for runtime consumption
        match serde_yaml::to_string(&runtime_callbacks) {
            Ok(yaml) => {
                println!("cargo:rustc-env=IMIX_CALLBACKS={}", yaml);
                println!(
                    "cargo:warning=Setting IMIX_CALLBACKS with {} callback(s)",
                    callbacks.len()
                );

                // Find first https:// callback to set as IMIX_CALLBACK_URI
                let https_callback_runtime = runtime_callbacks
                    .iter()
                    .find(|cb| cb.uri.to_lowercase().starts_with("https://"));

                if let Some(https_cb) = https_callback_runtime {
                    println!("cargo:rustc-env=IMIX_CALLBACK_URI={}", https_cb.uri);
                    println!(
                        "cargo:warning=Setting IMIX_CALLBACK_URI={} (first https:// callback)",
                        https_cb.uri
                    );
                } else {
                    println!("cargo:warning=No https:// callback found, IMIX_CALLBACK_URI not set");
                }
            }
            Err(e) => {
                println!("cargo:warning=Failed to serialize callbacks to YAML: {}", e);
            }
        }
    }

    // Set other configuration options
    if let Some(ref server_pubkey) = config.server_pubkey {
        println!("cargo:rustc-env=IMIX_SERVER_PUBKEY={}", server_pubkey);
        println!("cargo:warning=Setting IMIX_SERVER_PUBKEY=(redacted for security)");
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
                "win_service" => println!("cargo:rustc-cfg=feature=\"win_service\""),
                _ => println!("cargo:warning=Unknown feature: {}", feature),
            }
        }
    }
}

fn main() {
    // Check for legacy environment variables
    let has_legacy_callback_uri = env::var("IMIX_CALLBACK_URI").is_ok();
    let has_legacy_callback_interval = env::var("IMIX_CALLBACK_INTERVAL").is_ok();
    let has_legacy_retry_interval = env::var("IMIX_RETRY_INTERVAL").is_ok();
    let has_legacy_proxy_uri = env::var("IMIX_PROXY_URI").is_ok();
    let has_any_legacy = has_legacy_callback_uri
        || has_legacy_callback_interval
        || has_legacy_retry_interval
        || has_legacy_proxy_uri;

    // Try to read YAML configuration from environment variable
    if let Ok(yaml_content) = env::var("IMIX_BUILD_CONFIG") {
        println!("cargo:warning=Found IMIX_BUILD_CONFIG environment variable");
        println!("cargo:rerun-if-env-changed=IMIX_BUILD_CONFIG");

        // Error if both YAML config and legacy env vars are set
        if has_any_legacy {
            panic!(
                "Cannot use both IMIX_BUILD_CONFIG and legacy environment variables. \
                Found IMIX_BUILD_CONFIG along with one or more of: \
                IMIX_CALLBACK_URI, IMIX_CALLBACK_INTERVAL, IMIX_RETRY_INTERVAL, IMIX_PROXY_URI. \
                Please use either the YAML configuration (IMIX_BUILD_CONFIG) or the individual \
                environment variables, but not both."
            );
        }

        match parse_yaml_build_config(&yaml_content) {
            Ok(config) => {
                println!("cargo:warning=Successfully parsed YAML build configuration");
                apply_build_config(&config);
            }
            Err(e) => {
                panic!("Failed to parse YAML build configuration: {}", e);
            }
        }
    } else {
        println!("cargo:warning=No IMIX_BUILD_CONFIG environment variable found");
        if has_any_legacy {
            println!("cargo:warning=Using legacy individual environment variables for build configuration");
        } else {
            println!("cargo:warning=No configuration provided. Using defaults.");
        }
    }

    // Windows-specific build configuration
    #[cfg(target_os = "windows")]
    static_vcruntime::metabuild();
}
