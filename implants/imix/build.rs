use serde::Deserialize;
use std::env;
use std::path::PathBuf;
use which::which;

// ---- YAML config types (for IMIX_CONFIG parsing) ----

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
    server_pubkey: Option<String>,
}

struct YamlConfigResult {
    upstream_uri: Option<String>,
    server_pubkey: Option<String>,
}

fn parse_yaml_config() -> Result<Option<YamlConfigResult>, Box<dyn std::error::Error>> {
    let config_yaml = match std::env::var("IMIX_CONFIG") {
        Ok(yaml_content) => yaml_content,
        Err(_) => {
            println!("cargo:warning=IMIX_CONFIG not set, skipping YAML config parsing");
            return Ok(None);
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

    if let Some(ref pubkey) = config.server_pubkey {
        println!("cargo:rustc-env=IMIX_SERVER_PUBKEY={}", pubkey);
        println!("cargo:warning=Using server_pubkey from YAML config");
    }

    println!(
        "cargo:warning=Successfully parsed YAML config with {} transport(s)",
        config.transports.len()
    );

    let upstream_uri = config.transports.first().map(|t| t.uri.clone());

    Ok(Some(YamlConfigResult {
        upstream_uri,
        server_pubkey: config.server_pubkey,
    }))
}

fn get_pub_key(yaml_config: Option<YamlConfigResult>) {
    if let Some(ref config) = yaml_config {
        if config.server_pubkey.is_some() {
            println!("cargo:warning=Server pubkey provided via YAML config, skipping fetch");
            return;
        }
    }

    if std::env::var("IMIX_SERVER_PUBKEY").is_ok() {
        println!("cargo:warning=IMIX_SERVER_PUBKEY already set, skipping fetch");
        return;
    }

    let callback_uri = yaml_config
        .and_then(|c| c.upstream_uri)
        .or_else(|| std::env::var("IMIX_CALLBACK_URI").ok())
        .unwrap_or_else(|| "http://127.0.0.1:8000".to_string());

    let base_uri = callback_uri
        .split(';')
        .next()
        .unwrap_or(&callback_uri)
        .trim()
        .split('?')
        .next()
        .unwrap_or(&callback_uri);

    let status_url = format!("{}/status", base_uri);

    let client = match reqwest::blocking::Client::builder().http1_only().build() {
        Ok(c) => c,
        Err(e) => {
            println!("cargo:warning=Failed to build HTTP client: {}", e);
            return;
        }
    };
    let response = match client.get(&status_url).send() {
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

    println!("cargo:rustc-env=IMIX_SERVER_PUBKEY={}", pubkey);
    println!(
        "cargo:warning=Successfully fetched server public key from {}",
        status_url
    );
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
        let mut error_msg = String::from(
            "Configuration error: Cannot use both DSN query parameters and legacy environment variables.\n",
        );
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
    #[cfg(target_os = "windows")]
    static_vcruntime::metabuild();

    if std::env::var("CARGO_FEATURE_TOKIO_CONSOLE").is_ok() {
        println!("cargo:rustc-cfg=tokio_unstable");
    }

    println!("cargo:rerun-if-env-changed=IMIX_CONFIG");
    println!("cargo:rerun-if-env-changed=IMIX_CALLBACK_URI");
    println!("cargo:rerun-if-env-changed=IMIX_CALLBACK_INTERVAL");
    println!("cargo:rerun-if-env-changed=IMIX_SERVER_PUBKEY");
    println!("cargo:rerun-if-env-changed=IMIX_DEBUG");
    println!("cargo:rerun-if-env-changed=PROTOC");

    let profile = std::env::var("PROFILE").unwrap_or_default();
    let imix_debug = std::env::var("IMIX_DEBUG").unwrap_or_default();

    if profile == "debug" || imix_debug == "tomes" || imix_debug == "all" {
        println!("cargo:rustc-cfg=feature=\"print_debug_tome\"");
    }

    if profile == "debug" || imix_debug == "all" {
        println!("cargo:rustc-cfg=feature=\"print_debug\"");
    }

    // YAML config handling — emits IMIX_CALLBACK_URI and IMIX_SERVER_PUBKEY from YAML when set.
    let yaml_config = parse_yaml_config()?;

    // DSN legacy-env validation
    validate_dsn_config()?;

    // Auto-fetch server pubkey from Tavern /status when not already provided.
    get_pub_key(yaml_config);

    // Keep existing protoc-detection log line for consistency, but proto generation itself
    // lives in pb/build.rs.
    match env::var_os("PROTOC")
        .map(PathBuf::from)
        .or_else(|| which("protoc").ok())
    {
        Some(_) => println!("Found protoc (pb crate will generate protos)"),
        None => {
            println!("WARNING: Failed to locate protoc");
        }
    }

    Ok(())
}
