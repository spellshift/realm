use imix_config::{
    find_first_https_callback, parse_yaml_build_config, prepare_callbacks, validate_config,
    ImixBuildConfig,
};
use std::env;

/// Apply build configuration by setting cargo environment variables
fn apply_build_config(config: &ImixBuildConfig) {
    // Validate configuration
    if let Err(e) = validate_config(config) {
        panic!("cargo:error={}", e);
    }

    // Handle callbacks
    if let Some(ref callbacks) = config.callbacks {
        // Prepare callbacks for runtime by encoding doh_provider in URI query params
        let runtime_callbacks = prepare_callbacks(callbacks);

        // Serialize callbacks list as YAML for runtime consumption
        match serde_yaml::to_string(&runtime_callbacks) {
            Ok(yaml) => {
                println!("cargo:rustc-env=IMIX_CALLBACKS={}", yaml);
                println!(
                    "cargo:warning=Setting IMIX_CALLBACKS with {} callback(s)",
                    callbacks.len()
                );

                // Find first https:// callback to set as IMIX_CALLBACK_URI
                if let Some(https_cb) = find_first_https_callback(&runtime_callbacks) {
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
