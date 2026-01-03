use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use which::which;

fn get_pub_key() {
    // Check if IMIX_SERVER_PUBKEY is already set
    if std::env::var("IMIX_SERVER_PUBKEY").is_ok() {
        println!("cargo:warning=IMIX_SERVER_PUBKEY already set, skipping fetch");
        return;
    }

    // Get the callback URI from environment variable, default to http://127.0.0.1:8000
    let callback_uri =
        std::env::var("IMIX_CALLBACK_URI").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());

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

fn build_extra_vars() -> Result<(), Box<dyn std::error::Error>> {
    let mut res = HashMap::new();
    for (key, value) in std::env::vars() {
        if key.starts_with("IMIX_TRANSPORT_EXTRA_") {
            println!("{}", key);
            match key.strip_prefix("IMIX_TRANSPORT_EXTRA_") {
                Some(k) => {
                    let suffixed_key = String::from(k);
                    res.insert(suffixed_key, value);
                }
                None => panic!("failed to strip prefix"),
            }
        }
    }
    let res_str = serde_json::to_string(&res)?;
    println!("cargo:rustc-env=IMIX_TRANSPORT_EXTRA={}", res_str);
    Ok(())
}

fn validate_dsn_config() -> Result<(), Box<dyn std::error::Error>> {
    // Check if IMIX_CALLBACK_URI contains query parameters
    let callback_uri = std::env::var("IMIX_CALLBACK_URI").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
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
    validate_dsn_config()?;
    get_pub_key();
    build_extra_vars()?;

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
    match tonic_build::configure()
        .out_dir("./src/generated/")
        .codec_path("crate::xchacha::ChachaCodec")
        .build_client(false)
        .build_server(false)
        .compile(
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
    match tonic_build::configure()
        .out_dir("./src/generated/")
        .codec_path("crate::xchacha::ChachaCodec")
        .build_client(false)
        .build_server(false)
        .compile(
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
    match tonic_build::configure()
        .out_dir("./src/generated/")
        .codec_path("crate::xchacha::ChachaCodec")
        .build_client(false)
        .build_server(false)
        .compile(&["trace.proto"], &["../../../tavern/portals/proto/"])
    {
        Err(err) => {
            println!("WARNING: Failed to compile portal protos: {}", err);
            panic!("{}", err);
        }
        Ok(_) => println!("generated portal trace protos"),
    };

    // Build C2 Protos
    match tonic_build::configure()
        .out_dir("./src/generated")
        .codec_path("crate::xchacha::ChachaCodec")
        .build_server(false)
        .extern_path(".eldritch", "crate::eldritch")
        .compile(
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
    match tonic_build::configure()
        .out_dir("./src/generated")
        .build_server(false)
        .build_client(false)
        .compile(
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
