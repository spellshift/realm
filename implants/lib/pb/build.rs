use serde::Deserialize;
use std::env;
use std::path::PathBuf;
use which::which;

#[derive(Debug, Deserialize)]
struct TransportConfig {
    #[serde(rename = "URI")]
    uri: String,
    #[serde(rename = "type")]
    transport_type: String,
    extra: String,
    #[serde(default)]
    interval: Option<u64>,
    #[serde(default)]
    jitter: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct YamlConfig {
    transports: Vec<TransportConfig>,
    #[serde(default)]
    server_pubkey: Option<String>,
}

/// Result of parsing YAML config, containing values needed by other build steps
struct YamlConfigResult {
    /// The first transport URI (used for fetching pubkey)
    upstream_uri: Option<String>,
    /// Server public key if specified in config
    server_pubkey: Option<String>,
}

fn parse_yaml_config() -> Result<Option<YamlConfigResult>, Box<dyn std::error::Error>> {
    // Check if IMIX_CONFIG is set
    let config_yaml = match std::env::var("IMIX_CONFIG") {
        Ok(yaml_content) => yaml_content,
        Err(_) => {
            println!("cargo:warning=IMIX_CONFIG not set, skipping YAML config parsing");
            return Ok(None);
        }
    };

    // Check that other configuration options are not set
    let has_callback_uri = std::env::var("IMIX_CALLBACK_URI").is_ok();
    let has_callback_interval = std::env::var("IMIX_CALLBACK_INTERVAL").is_ok();
    let has_transport_extra = std::env::vars().any(|(k, _)| k.starts_with("IMIX_TRANSPORT_EXTRA_"));

    if has_callback_uri || has_callback_interval || has_transport_extra {
        let mut error_msg = String::from(
            "Configuration error: Cannot use IMIX_CONFIG with other configuration options.\n",
        );
        error_msg.push_str(
            "When IMIX_CONFIG is set, all configuration must be done through the YAML file.\n",
        );
        error_msg.push_str("Found one or more of:\n");

        if has_callback_uri {
            error_msg.push_str("  - IMIX_CALLBACK_URI\n");
        }
        if has_callback_interval {
            error_msg.push_str("  - IMIX_CALLBACK_INTERVAL\n");
        }
        if has_transport_extra {
            error_msg.push_str("  - IMIX_TRANSPORT_EXTRA_*\n");
        }

        error_msg.push_str(
            "\nPlease use ONLY the YAML config file OR use environment variables, but not both.",
        );

        return Err(error_msg.into());
    }

    // Parse the YAML config
    let config: YamlConfig = serde_yaml::from_str(&config_yaml)
        .map_err(|e| format!("Failed to parse YAML config: {}", e))?;

    // Validate that we have at least one transport
    if config.transports.is_empty() {
        return Err("YAML config must contain at least one transport".into());
    }

    // Build DSN string from transports
    let mut dsn_parts = Vec::new();

    for transport in &config.transports {
        // Validate transport type
        let transport_type_lower = transport.transport_type.to_lowercase();
        if !["grpc", "http1", "dns"].contains(&transport_type_lower.as_str()) {
            return Err(format!(
                "Invalid transport type '{}'. Must be one of: GRPC, http1, DNS",
                transport.transport_type
            )
            .into());
        }

        // Validate that extra is valid JSON
        if !transport.extra.is_empty() {
            serde_json::from_str::<serde_json::Value>(&transport.extra).map_err(|e| {
                format!(
                    "Invalid JSON in 'extra' field for transport '{}': {}",
                    transport.uri, e
                )
            })?;
        }

        // Error if URI already contains query parameters
        if transport.uri.contains('?') {
            return Err(format!("URI '{}' already contains query parameters. Query parameters should not be present in the URI field.", transport.uri).into());
        }

        // Build DSN part with correct schema and query parameters
        let mut dsn_part = transport.uri.clone();

        // Add query parameters
        dsn_part.push('?');
        let mut params = Vec::new();

        // Add interval if present
        if let Some(interval) = transport.interval {
            params.push(format!("interval={}", interval));
        }

        // Add jitter if present
        if let Some(jitter) = transport.jitter {
            params.push(format!("jitter={}", jitter));
        }

        // Add extra as query parameter if not empty
        if !transport.extra.is_empty() {
            let encoded_extra = urlencoding::encode(&transport.extra);
            params.push(format!("extra={}", encoded_extra));
        }

        if !params.is_empty() {
            dsn_part.push_str(&params.join("&"));
        } else {
            // Remove the trailing '?' if no params were added
            dsn_part.pop();
        }

        dsn_parts.push(dsn_part);
    }

    // Join all DSN parts with semicolons
    let dsn = dsn_parts.join(";");

    // Emit the DSN configuration
    println!("cargo:rustc-env=IMIX_CALLBACK_URI={}", dsn);

    // Emit server_pubkey if present
    if let Some(ref pubkey) = config.server_pubkey {
        println!("cargo:rustc-env=IMIX_SERVER_PUBKEY={}", pubkey);
        println!("cargo:warning=Using server_pubkey from YAML config");
    }

    println!(
        "cargo:warning=Successfully parsed YAML config with {} transport(s)",
        config.transports.len()
    );

    // Extract the first transport URI for pubkey fetching
    let upstream_uri = config.transports.first().map(|t| t.uri.clone());

    Ok(Some(YamlConfigResult {
        upstream_uri,
        server_pubkey: config.server_pubkey,
    }))
}

fn get_pub_key(yaml_config: Option<YamlConfigResult>) {
    // Check if server pubkey was provided via YAML config
    if let Some(ref config) = yaml_config {
        if config.server_pubkey.is_some() {
            // Already emitted in parse_yaml_config, no need to fetch
            println!("cargo:warning=Server pubkey provided via YAML config, skipping fetch");
            return;
        }
    }

    // Check if IMIX_SERVER_PUBKEY is already set via env var
    if std::env::var("IMIX_SERVER_PUBKEY").is_ok() {
        println!("cargo:warning=IMIX_SERVER_PUBKEY already set, skipping fetch");
        return;
    }

    // Get the callback URI: prefer YAML config upstream, then env var, then default
    let callback_uri = yaml_config
        .and_then(|c| c.upstream_uri)
        .or_else(|| std::env::var("IMIX_CALLBACK_URI").ok())
        .unwrap_or_else(|| "http://127.0.0.1:8000".to_string());

    // Extract the first URI from semicolon-separated list and strip query parameters
    let base_uri = callback_uri
        .split(';')
        .next()
        .unwrap_or(&callback_uri)
        .trim()
        .split('?')
        .next()
        .unwrap_or(&callback_uri);

    // Construct the status endpoint URL
    let status_url = format!("{}/status", base_uri);

    // Make a GET request to /status
    let response = match reqwest::blocking::get(&status_url) {
        Ok(resp) => resp,
        Err(e) => {
            println!("cargo:warning=Failed to connect to {}: {}", status_url, e);
            return;
        }
    };

    if !response.status().is_success() {
        println!(
            "cargo:warning=Failed to fetch status from {}: HTTP {}",
            status_url,
            response.status()
        );
        return;
    }

    let json = match response.json::<serde_json::Value>() {
        Ok(json) => json,
        Err(e) => {
            println!(
                "cargo:warning=Failed to parse JSON response from {}: {}",
                status_url, e
            );
            return;
        }
    };

    let pubkey = match json.get("Pubkey").and_then(|v| v.as_str()) {
        Some(key) => key,
        None => {
            println!(
                "cargo:warning=Pubkey field not found in response from {}",
                status_url
            );
            return;
        }
    };

    // Set the IMIX_SERVER_PUBKEY environment variable for the build
    println!("cargo:rustc-env=IMIX_SERVER_PUBKEY={}", pubkey);
    println!(
        "cargo:warning=Successfully fetched server public key from {}",
        status_url
    );
}

fn validate_dsn_config() -> Result<(), Box<dyn std::error::Error>> {
    // Skip validation if YAML config is being used
    // (parse_yaml_config already handles validation in that case)
    if std::env::var("IMIX_CONFIG").is_ok() {
        return Ok(());
    }

    // Check if IMIX_CALLBACK_URI contains query parameters
    let callback_uri =
        std::env::var("IMIX_CALLBACK_URI").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
    let has_query_params = callback_uri.contains('?');

    // Check if legacy config environment variables are set
    let has_callback_interval = std::env::var("IMIX_CALLBACK_INTERVAL").is_ok();
    let has_transport_extra = std::env::vars().any(|(k, _)| k.starts_with("IMIX_TRANSPORT_EXTRA_"));

    // If DSN has query parameters AND legacy config is set, this is an error
    if has_query_params && (has_callback_interval || has_transport_extra) {
        let mut error_msg = String::from("Configuration error: Cannot use both DSN query parameters and legacy environment variables.\n");
        error_msg.push_str("Found query parameters in IMIX_CALLBACK_URI and one or more of:\n");

        if has_callback_interval {
            error_msg.push_str("  - IMIX_CALLBACK_INTERVAL\n");
        }
        if has_transport_extra {
            error_msg.push_str("  - IMIX_TRANSPORT_EXTRA_*\n");
        }

        error_msg.push_str("\nPlease use ONLY DSN query parameters (e.g., https://example.com?interval=10&extra={...})\n");
        error_msg.push_str("OR use legacy environment variables, but not both.");

        return Err(error_msg.into());
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell Cargo to rerun this build script if these env vars change
    // This fixes the issue where changing IMIX_CONFIG doesn't trigger a rebuild
    println!("cargo:rerun-if-env-changed=IMIX_CONFIG");
    println!("cargo:rerun-if-env-changed=IMIX_CALLBACK_URI");
    println!("cargo:rerun-if-env-changed=IMIX_CALLBACK_INTERVAL");
    println!("cargo:rerun-if-env-changed=IMIX_SERVER_PUBKEY");
    println!("cargo:rerun-if-env-changed=PROTOC");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");

    // Parse YAML config if present (this will emit IMIX_CALLBACK_URI if successful)
    let yaml_config = parse_yaml_config()?;

    // Validate DSN config (skips if YAML config was used)
    validate_dsn_config()?;

    get_pub_key(yaml_config);

    // Skip if no `protoc` can be found
    match env::var_os("PROTOC")
        .map(PathBuf::from)
        .or_else(|| which("protoc").ok())
    {
        Some(_) => println!("Found protoc, protos will be generated"),
        None => {
            println!("WARNING: Failed to locate protoc, protos will not be generated");
            return Ok(());
        }
    }

    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let is_wasm = target_arch == "wasm32";

    // Build Eldritch Proto
    let mut eldritch_config = tonic_prost_build::configure();
    eldritch_config = eldritch_config.out_dir("./src/generated/");
    if !is_wasm {
        eldritch_config = eldritch_config.codec_path("crate::xchacha::ChachaCodec");
    } else {
        eldritch_config = eldritch_config.build_client(false);
        eldritch_config = eldritch_config.build_server(false);
    }
    match eldritch_config.compile_protos(
        &["eldritch.proto"],
        &[
            "../../../tavern/internal/c2/proto/",
            "../../../tavern/portals/proto/",
        ],
    ) {
        Err(err) => {
            println!("WARNING: Failed to compile eldritch protos: {}", err);
            panic!("{}", err);
        }
        Ok(_) => println!("generated eldritch protos"),
    };

    // Build Portal Protos
    let mut portal_config = tonic_prost_build::configure();
    portal_config = portal_config.out_dir("./src/generated/");
    if !is_wasm {
        portal_config = portal_config.codec_path("crate::xchacha::ChachaCodec");
    } else {
        portal_config = portal_config.build_client(false);
        portal_config = portal_config.build_server(false);
    }
    match portal_config.compile_protos(
        &["portal.proto"],
        &[
            "../../../tavern/internal/c2/proto/",
            "../../../tavern/portals/proto/",
        ],
    ) {
        Err(err) => {
            println!("WARNING: Failed to compile portal protos: {}", err);
            panic!("{}", err);
        }
        Ok(_) => println!("generated portal protos"),
    };

    let mut trace_config = tonic_prost_build::configure();
    trace_config = trace_config.out_dir("./src/generated/");
    if !is_wasm {
        trace_config = trace_config.codec_path("crate::xchacha::ChachaCodec");
    }
    trace_config = trace_config.build_client(false);
    trace_config = trace_config.build_server(false);
    match trace_config.compile_protos(&["trace.proto"], &["../../../tavern/portals/proto/"]) {
        Err(err) => {
            println!("WARNING: Failed to compile portal protos: {}", err);
            panic!("{}", err);
        }
        Ok(_) => println!("generated portal trace protos"),
    };

    // Build C2 Protos
    let mut c2_config = tonic_prost_build::configure();
    c2_config = c2_config.out_dir("./src/generated");
    if !is_wasm {
        c2_config = c2_config.codec_path("crate::xchacha::ChachaCodec");
    } else {
        c2_config = c2_config.build_client(false);
        c2_config = c2_config.build_server(false);
    }
    c2_config = c2_config.build_server(false);
    c2_config = c2_config.extern_path(".eldritch", "crate::eldritch");
    match c2_config.compile_protos(
        &["c2.proto"],
        &[
            "../../../tavern/internal/c2/proto/",
            "../../../tavern/portals/proto/",
        ],
    ) {
        Err(err) => {
            println!("WARNING: Failed to compile c2 protos: {}", err);
            panic!("{}", err);
        }
        Ok(_) => println!("generated c2 protos"),
    };

    // Build DNS Protos (no encryption codec - used for transport layer only)
    match tonic_prost_build::configure()
        .out_dir("./src/generated")
        .build_server(false)
        .build_client(false)
        .compile_protos(
            &["dns.proto"],
            &[
                "../../../tavern/internal/c2/proto/",
                "../../../tavern/portals/proto/",
            ],
        ) {
        Err(err) => {
            println!("WARNING: Failed to compile dns protos: {}", err);
            panic!("{}", err);
        }
        Ok(_) => println!("generated dns protos"),
    };

    Ok(())
}
