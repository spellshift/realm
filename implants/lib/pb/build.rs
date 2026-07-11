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
    #[serde(default)]
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
    #[allow(dead_code)]
    server_pubkey: Option<String>,
}

fn parse_yaml_config() -> Result<(), Box<dyn std::error::Error>> {
    let config_yaml = match std::env::var("IMIX_CONFIG") {
        Ok(yaml_content) => yaml_content,
        Err(_) => {
            println!("cargo:warning=IMIX_CONFIG not set, skipping YAML config parsing");
            return Ok(());
        }
    };

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

    let config: YamlConfig = serde_yaml::from_str(&config_yaml)
        .map_err(|e| format!("Failed to parse YAML config: {}", e))?;

    if config.transports.is_empty() {
        return Err("YAML config must contain at least one transport".into());
    }

    let mut dsn_parts = Vec::new();

    for transport in &config.transports {
        let transport_type_lower = transport.transport_type.to_lowercase();
        if !["grpc", "http1", "dns", "icmp", "tcp_bind", "quic"]
            .contains(&transport_type_lower.as_str())
        {
            return Err(format!(
                "Invalid transport type '{}'. Must be one of: GRPC, http1, DNS, tcp_bind, quic",
                transport.transport_type
            )
            .into());
        }

        if !transport.extra.is_empty() {
            serde_json::from_str::<serde_json::Value>(&transport.extra).map_err(|e| {
                format!(
                    "Invalid JSON in 'extra' field for transport '{}': {}",
                    transport.uri, e
                )
            })?;
        }

        if transport.uri.contains('?') {
            return Err(format!("URI '{}' already contains query parameters. Query parameters should not be present in the URI field.", transport.uri).into());
        }

        let mut dsn_part = transport.uri.clone();
        dsn_part.push('?');
        let mut params = Vec::new();

        if let Some(interval) = transport.interval {
            params.push(format!("interval={}", interval));
        }

        if let Some(jitter) = transport.jitter {
            params.push(format!("jitter={}", jitter));
        }

        params.push(format!("type={}", transport_type_lower));

        if !transport.extra.is_empty() {
            let encoded_extra = urlencoding::encode(&transport.extra);
            params.push(format!("extra={}", encoded_extra));
        }

        if !params.is_empty() {
            dsn_part.push_str(&params.join("&"));
        } else {
            dsn_part.pop();
        }

        dsn_parts.push(dsn_part);
    }

    let dsn = dsn_parts.join(";");

    println!("cargo:rustc-env=IMIX_CALLBACK_URI={}", dsn);

    // NOTE: server_pubkey from YAML is intentionally not handled here.
    // SERVER_PUBKEY configuration is owned by the imix crate's build.rs.

    println!(
        "cargo:warning=Successfully parsed YAML config with {} transport(s)",
        config.transports.len()
    );

    Ok(())
}

fn validate_dsn_config() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("IMIX_CONFIG").is_ok() {
        return Ok(());
    }

    let callback_uri =
        std::env::var("IMIX_CALLBACK_URI").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
    let has_query_params = callback_uri.contains('?');

    let has_callback_interval = std::env::var("IMIX_CALLBACK_INTERVAL").is_ok();
    let has_transport_extra = std::env::vars().any(|(k, _)| k.starts_with("IMIX_TRANSPORT_EXTRA_"));

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
    println!("cargo:rerun-if-env-changed=IMIX_CONFIG");
    println!("cargo:rerun-if-env-changed=IMIX_CALLBACK_URI");
    println!("cargo:rerun-if-env-changed=IMIX_CALLBACK_INTERVAL");
    println!("cargo:rerun-if-env-changed=PROTOC");
    println!("cargo:rerun-if-env-changed=IMIX_DEBUG");
    let profile = std::env::var("PROFILE").unwrap_or_default();
    let imix_debug = std::env::var("IMIX_DEBUG").unwrap_or_default();

    if profile == "debug" || imix_debug == "tomes" || imix_debug == "all" {
        println!("cargo:rustc-cfg=feature=\"print_debug_tome\"");
    }

    if profile == "debug" || imix_debug == "all" {
        println!("cargo:rustc-cfg=feature=\"print_debug\"");
    }

    // Parse YAML config if present (this will emit IMIX_CALLBACK_URI if successful).
    // NOTE: SERVER_PUBKEY handling has been moved to imix/build.rs.
    parse_yaml_config()?;

    // Validate DSN config (skips if YAML config was used)
    validate_dsn_config()?;

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

    // Build Eldritch Proto
    match tonic_prost_build::configure()
        .out_dir("./src/generated/")
        .codec_path("crate::xchacha::ChachaCodec")
        .build_client(false)
        .build_server(false)
        .compile_protos(
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
    match tonic_prost_build::configure()
        .out_dir("./src/generated/")
        .codec_path("crate::xchacha::ChachaCodec")
        .build_client(false)
        .build_server(false)
        .compile_protos(
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
    match tonic_prost_build::configure()
        .out_dir("./src/generated/")
        .codec_path("crate::xchacha::ChachaCodec")
        .build_client(false)
        .build_server(false)
        .compile_protos(&["trace.proto"], &["../../../tavern/portals/proto/"])
    {
        Err(err) => {
            println!("WARNING: Failed to compile portal protos: {}", err);
            panic!("{}", err);
        }
        Ok(_) => println!("generated portal trace protos"),
    };

    // Build C2 Protos
    match tonic_prost_build::configure()
        .out_dir("./src/generated")
        .codec_path("crate::xchacha::ChachaCodec")
        .build_server(false)
        .extern_path(".eldritch", "crate::eldritch")
        .compile_protos(
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

    // Build Conv Protos (no encryption codec - shared conversation protocol)
    match tonic_prost_build::configure()
        .out_dir("./src/generated")
        .build_server(false)
        .build_client(false)
        .compile_protos(
            &["conv.proto"],
            &[
                "../../../tavern/internal/c2/proto/",
                "../../../tavern/portals/proto/",
            ],
        ) {
        Err(err) => {
            println!("WARNING: Failed to compile conv protos: {}", err);
            panic!("{}", err);
        }
        Ok(_) => println!("generated conv protos"),
    };

    Ok(())
}
