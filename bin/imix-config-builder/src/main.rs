use anyhow::{anyhow, bail, Context, Result};
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    server_pubkey: String,
    interval: u64,
    retry_interval: u64,
    callbacks: Vec<Callback>,
    #[serde(default)]
    run_once: Option<bool>,
    features: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Callback {
    #[serde(rename = "type")]
    callback_type: String,
    uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    doh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    proxy_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_type: Option<String>,
}

fn validate_base64(s: &str, expected_len: usize) -> Result<()> {
    let decoded = BASE64_STANDARD
        .decode(s)
        .context("Invalid base64 encoding")?;

    if decoded.len() != expected_len {
        bail!(
            "Base64 string has incorrect length: expected {} bytes, got {}",
            expected_len,
            decoded.len()
        );
    }

    Ok(())
}

fn validate_uri(uri_str: &str, callback_type: &str) -> Result<()> {
    let uri = url::Url::parse(uri_str).with_context(|| format!("Invalid URI: {}", uri_str))?;

    match callback_type {
        "grpc" => {
            if !["http", "https"].contains(&uri.scheme()) {
                bail!(
                    "gRPC callback URI must use http or https scheme, got: {}",
                    uri.scheme()
                );
            }
        }
        "http1" => {
            if !["http1", "https1"].contains(&uri.scheme()) {
                bail!(
                    "HTTP1 callback URI must use http1 or https1 scheme, got: {}",
                    uri.scheme()
                );
            }
        }
        "dns" => {
            if uri.scheme() != "dns" {
                bail!(
                    "DNS callback URI must use dns scheme, got: {}",
                    uri.scheme()
                );
            }
        }
        _ => bail!("Unknown callback type: {}", callback_type),
    }

    Ok(())
}

fn validate_callback(callback: &Callback) -> Result<()> {
    // Validate callback type
    if !["grpc", "http1", "dns"].contains(&callback.callback_type.as_str()) {
        bail!(
            "Invalid callback type: {}. Must be one of: grpc, http1, dns",
            callback.callback_type
        );
    }

    // Validate URI according to type
    validate_uri(&callback.uri, &callback.callback_type)?;

    // DNS-specific validations
    if callback.callback_type == "dns" {
        if callback.domain.is_none() {
            bail!("DNS callback requires 'domain' field");
        }
        if callback.query_type.is_none() {
            bail!("DNS callback requires 'query_type' field");
        }
    }

    // gRPC-specific validations
    if callback.callback_type == "grpc" {
        if let Some(ref doh) = callback.doh {
            if !["cloudflare", "google", "quad9"].contains(&doh.as_str()) {
                bail!(
                    "Invalid DoH provider: {}. Must be one of: cloudflare, google, quad9",
                    doh
                );
            }
        }

        if let Some(ref proxy) = callback.proxy_uri {
            url::Url::parse(proxy).with_context(|| format!("Invalid proxy URI: {}", proxy))?;
        }
    } else {
        // Non-gRPC callbacks should not have DoH set
        if callback.doh.is_some() {
            bail!("DoH provider can only be set for gRPC callbacks");
        }
    }

    Ok(())
}

fn validate_config(config: &Config) -> Result<()> {
    // Validate server_pubkey
    validate_base64(&config.server_pubkey, 32)
        .context("server_pubkey must be a valid base64 string of 32 bytes (44 characters)")?;

    // Validate callbacks
    if config.callbacks.is_empty() {
        bail!("At least one callback must be specified");
    }

    for (i, callback) in config.callbacks.iter().enumerate() {
        validate_callback(callback).with_context(|| format!("Invalid callback at index {}", i))?;
    }

    // Validate intervals
    if config.interval == 0 {
        bail!("interval must be a positive integer");
    }
    if config.retry_interval == 0 {
        bail!("retry_interval must be a positive integer");
    }

    // Validate features
    for feature in &config.features {
        if !["grpc", "http1", "dns"].contains(&feature.as_str()) {
            bail!(
                "Invalid feature: {}. Must be one of: grpc, http1, dns",
                feature
            );
        }
    }

    Ok(())
}

fn check_legacy_env_vars() -> Result<()> {
    let has_legacy = env::var("IMIX_CALLBACK_URI").is_ok()
        || env::var("IMIX_PROXY_URI").is_ok()
        || env::var("IMIX_CALLBACK_INTERVAL").is_ok()
        || env::var("IMIX_RETRY_INTERVAL").is_ok();

    if has_legacy {
        bail!(
            "Legacy environment variables (IMIX_CALLBACK_URI, IMIX_PROXY_URI, \
            IMIX_CALLBACK_INTERVAL, IMIX_RETRY_INTERVAL) cannot be set when using \
            IMIX_CONFIG. Please use only IMIX_CONFIG for configuration."
        );
    }

    Ok(())
}

fn emit_build_directives(config: &Config) -> Result<()> {
    // Emit server public key
    println!(
        "cargo:rustc-env=IMIX_SERVER_PUBKEY={}",
        config.server_pubkey
    );

    // Emit intervals
    println!("cargo:rustc-env=IMIX_CALLBACK_INTERVAL={}", config.interval);
    println!(
        "cargo:rustc-env=IMIX_RETRY_INTERVAL={}",
        config.retry_interval
    );

    // Emit run_once if set
    if config.run_once.unwrap_or(false) {
        println!("cargo:rustc-env=IMIX_RUN_ONCE=true");
    }

    // Build callback URI list (semicolon-delimited)
    let callback_uris: Vec<String> = config
        .callbacks
        .iter()
        .map(|cb| {
            let mut uri = cb.uri.clone();

            // Add query parameters for DoH and proxy_uri
            let mut params = vec![];
            if let Some(ref doh) = cb.doh {
                params.push(format!("doh={}", doh));
            }
            if let Some(ref proxy) = cb.proxy_uri {
                params.push(format!("proxy_uri={}", urlencoding::encode(proxy)));
            }
            if let Some(ref domain) = cb.domain {
                params.push(format!("domain={}", domain));
            }
            if let Some(ref query_type) = cb.query_type {
                params.push(format!("query_type={}", query_type));
            }

            if !params.is_empty() {
                uri = format!("{}?{}", uri, params.join("&"));
            }

            uri
        })
        .collect();

    println!(
        "cargo:rustc-env=IMIX_CALLBACK_URI={}",
        callback_uris.join(";")
    );

    // Emit feature flags
    for feature in &config.features {
        println!("cargo:rustc-cfg=feature=\"{}\"", feature);
    }

    Ok(())
}

fn main() -> Result<()> {
    // Check if config is provided via environment variable
    let config_yaml = match env::var("IMIX_CONFIG") {
        Ok(yaml) => yaml,
        Err(_) => {
            // No config provided, exit silently (allows for legacy mode)
            return Ok(());
        }
    };

    // If config is provided, check that legacy env vars are not set
    check_legacy_env_vars()?;

    // Parse YAML config
    let config: Config =
        serde_yaml::from_str(&config_yaml).context("Failed to parse YAML configuration")?;

    // Validate config
    validate_config(&config).context("Configuration validation failed")?;

    // Emit cargo build directives
    emit_build_directives(&config)?;

    Ok(())
}
