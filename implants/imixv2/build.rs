use imix_config::{apply_build_config, parse_yaml_build_config};
use std::env;

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
