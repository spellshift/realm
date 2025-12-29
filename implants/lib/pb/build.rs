use anyhow::Result;
use std::env;
use std::path::PathBuf;
use which::which;

fn parse_transport_uri(uri: &str) -> Result<String> {
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

    Ok(base_uri)
}
fn get_pub_key() {
    // Check if IMIX_SERVER_PUBKEY is already set
    if std::env::var("IMIX_SERVER_PUBKEY").is_ok() {
        println!("cargo:warning=IMIX_SERVER_PUBKEY already set, skipping fetch");
        return;
    }

    // Get the callback URI from environment variable, default to http://127.0.0.1:8000
    let full_callback_uri =
        std::env::var("IMIX_CALLBACK_URI").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());

    let callback_uri = match parse_transport_uri(&full_callback_uri) {
        Ok(uri) => uri,
        Err(e) => {
            println!(
                "cargo:warning=Failed to parse IMIX_CALLBACK_URI '{}': {}",
                full_callback_uri, e
            );
            return;
        }
    };

    // Construct the status endpoint URL
    let status_url = format!("{}/status", callback_uri);

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    get_pub_key();

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
        .compile(&["eldritch.proto"], &["../../../tavern/internal/c2/proto"])
    {
        Err(err) => {
            println!("WARNING: Failed to compile eldritch protos: {}", err);
            panic!("{}", err);
        }
        Ok(_) => println!("generated eldritch protos"),
    };

    // Build C2 Protos
    match tonic_build::configure()
        .out_dir("./src/generated")
        .codec_path("crate::xchacha::ChachaCodec")
        .build_server(false)
        .extern_path(".eldritch", "crate::eldritch")
        .compile(&["c2.proto"], &["../../../tavern/internal/c2/proto/"])
    {
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
        .compile(&["dns.proto"], &["../../../tavern/internal/c2/proto/"])
    {
        Err(err) => {
            println!("WARNING: Failed to compile dns protos: {}", err);
            panic!("{}", err);
        }
        Ok(_) => println!("generated dns protos"),
    };

    Ok(())
}
