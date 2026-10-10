use anyhow::Context;
use guardrails::Guardrail;
use host_unique::HostIDSelector;
use std::sync::OnceLock;
use url::Url;
use uuid::Uuid;

use crate::c2::{AvailableTransports, Transport};

// ---------------------------------------------------------------------------
// Runtime-settable imix configuration — owned by imix, not pb.
// imix's startup calls `init_runtime_config(RuntimeImixConfig)` before any
// call to `Config::default_with_imix_version`. All other consumers (eldritch,
// golem, tests) use the defaults.
// ---------------------------------------------------------------------------

/// All imix-tunable values that were formerly expressed as compile-time
/// `option_env!` / `env!` constants inside this crate. Imix constructs this
/// from its own build.rs-emitted rustc-env and/or legacy env vars.
#[derive(Debug, Clone)]
pub struct RuntimeImixConfig {
    pub callback_uri: String,
    pub callback_interval: String,
    pub retry_interval: String,
    pub run_once: bool,
    pub transport_extra: String,
    pub unique_json: Option<String>,
    pub guardrails_json: Option<String>,
}

impl Default for RuntimeImixConfig {
    fn default() -> Self {
        Self {
            callback_uri: "http://127.0.0.1:8000".to_string(),
            callback_interval: "5".to_string(),
            retry_interval: "5".to_string(),
            run_once: false,
            transport_extra: String::new(),
            unique_json: None,
            guardrails_json: None,
        }
    }
}

static RUNTIME_CONFIG: OnceLock<RuntimeImixConfig> = OnceLock::new();

/// Called by imix at startup. No-op if already set (OnceLock).
pub fn init_runtime_config(cfg: RuntimeImixConfig) {
    let _ = RUNTIME_CONFIG.set(cfg);
}

fn runtime_config() -> RuntimeImixConfig {
    RUNTIME_CONFIG.get().cloned().unwrap_or_default()
}

// Convenience accessors (kept for tests / existing call sites that used consts).

pub fn callback_uri() -> String {
    runtime_config().callback_uri
}

pub fn callback_interval() -> String {
    runtime_config().callback_interval
}

pub fn retry_interval() -> String {
    runtime_config().retry_interval
}

pub fn run_once() -> bool {
    runtime_config().run_once
}

pub fn transport_extra() -> String {
    runtime_config().transport_extra
}

// Kept for API compatibility — callers that used the const directly now call these.
pub const CALLBACK_URI_DEFAULT: &str = "http://127.0.0.1:8000";
pub const CALLBACK_INTERVAL_DEFAULT: &str = "5";
pub const RETRY_INTERVAL_DEFAULT: &str = "5";

// For backward compatibility with any code that directly references the old
// compile-time constants (tests, etc.). These now read via runtime_config().
#[deprecated(note = "Use pb::config::callback_uri() / init_runtime_config instead")]
pub const CALLBACK_URI: &str = CALLBACK_URI_DEFAULT;
#[deprecated(note = "Use pb::config::callback_interval() instead")]
pub const CALLBACK_INTERVAL: &str = CALLBACK_INTERVAL_DEFAULT;
#[deprecated(note = "Use pb::config::retry_interval() instead")]
pub const RETRY_INTERVAL: &str = RETRY_INTERVAL_DEFAULT;
#[deprecated(note = "Use pb::config::transport_extra() instead")]
pub const DEFAULT_EXTRA_CONFIG: &str = "";
#[deprecated(note = "Use pb::config::run_once() instead")]
pub const RUN_ONCE: bool = false;

//TODO: Can this struct be removed?
/// Config holds values necessary to configure an Agent.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Config {
    #[prost(message, optional, tag = "1")]
    pub info: ::core::option::Option<crate::c2::Beacon>,
    #[prost(bool, tag = "2")]
    pub run_once: bool,
}

/// Determine transport type from URI scheme.
fn get_transport_type(uri: &str) -> crate::c2::transport::Type {
    match uri.split(":").next().unwrap_or("unspecified") {
        "dns" => crate::c2::transport::Type::TransportDns,
        "icmp" => crate::c2::transport::Type::TransportIcmp,
        "http1" => crate::c2::transport::Type::TransportHttp1,
        "https1" => crate::c2::transport::Type::TransportHttp1,
        "https" => crate::c2::transport::Type::TransportGrpc,
        "http" => crate::c2::transport::Type::TransportGrpc,
        "tcp" => crate::c2::transport::Type::TransportTcpBind,
        "quic" => crate::c2::transport::Type::TransportQuic,
        "quics" => crate::c2::transport::Type::TransportQuic,
        _ => crate::c2::transport::Type::TransportUnspecified,
    }
}

/// Parse URIs into Transport objects. Supports DSN format with query params:
/// interval, extra, jitter, type. Example:
/// `https://example.com?interval=10&extra={"key":"value"}&jitter=0.5`
pub fn parse_transports(uri_string: &str) -> Vec<Transport> {
    uri_string
        .split(';')
        .filter(|s| !s.trim().is_empty())
        .filter_map(|uri| parse_dsn(uri.trim()).ok())
        .collect()
}

/// Parse a single DSN URI into a Transport struct.
pub fn parse_dsn(uri: &str) -> anyhow::Result<Transport> {
    let parsed_url = Url::parse(uri).with_context(|| format!("Failed to parse URI '{}'", uri))?;

    let rt = runtime_config();
    let mut interval = parse_callback_interval_with(&rt.callback_interval)?;
    let mut extra = rt.transport_extra.to_lowercase();
    let mut jitter = 0.0_f32;
    let mut transport_type = get_transport_type(uri);

    for (key, value) in parsed_url.query_pairs() {
        match key.as_ref() {
            "interval" => {
                interval = value
                    .parse::<u64>()
                    .with_context(|| format!("Failed to parse interval parameter '{}'", value))?;
            }
            "extra" => {
                extra = value.to_lowercase();
            }
            "jitter" => {
                jitter = value
                    .parse::<f32>()
                    .with_context(|| format!("Failed to parse jitter parameter '{}'", value))?;
            }
            "type" => {
                transport_type = match value.to_lowercase().as_str() {
                    "grpc" => crate::c2::transport::Type::TransportGrpc,
                    "http1" => crate::c2::transport::Type::TransportHttp1,
                    "dns" => crate::c2::transport::Type::TransportDns,
                    "icmp" => crate::c2::transport::Type::TransportIcmp,
                    "tcp_bind" => crate::c2::transport::Type::TransportTcpBind,
                    "quic" => crate::c2::transport::Type::TransportQuic,
                    _ => crate::c2::transport::Type::TransportUnspecified,
                };
            }
            _ => {
                #[cfg(feature = "print_debug")]
                log::debug!("Ignoring unknown query parameter: {}", key);
            }
        }
    }

    let mut base_uri = parsed_url.clone();
    base_uri.set_query(None);

    Ok(Transport {
        uri: base_uri.to_string(),
        interval,
        r#type: transport_type as i32,
        extra,
        jitter,
    })
}

fn parse_callback_interval_with(s: &str) -> anyhow::Result<u64> {
    s.parse::<u64>()
        .with_context(|| format!("Failed to parse callback interval constant '{}'", s))
}

#[allow(dead_code)]
fn parse_callback_interval() -> anyhow::Result<u64> {
    parse_callback_interval_with(&runtime_config().callback_interval)
}

fn parse_host_unique_selectors() -> Vec<Box<dyn HostIDSelector>> {
    let rt = runtime_config();
    let final_res = match rt.unique_json {
        Some(json) => {
            if let Some(res) = host_unique::from_imix_unique(json) {
                return res;
            } else {
                #[cfg(feature = "print_debug")]
                log::error!(
                    "Error parsing uniqueness string (should have been caught at build time)"
                );
                return host_unique::defaults();
            }
        }
        None => host_unique::defaults(),
    };
    final_res
}

fn parse_guardrails() -> Vec<Box<dyn Guardrail>> {
    let rt = runtime_config();
    let final_res = match rt.guardrails_json {
        Some(json) => {
            if let Some(res) = guardrails::from_imix_guardrails(json) {
                return res;
            } else {
                #[cfg(feature = "print_debug")]
                log::error!(
                    "Error parsing guardrails string (should have been caught at build time)"
                );
                return guardrails::defaults();
            }
        }
        None => guardrails::defaults(),
    };
    final_res
}

impl Config {
    pub fn default_with_imix_version(imix_version: &str) -> Self {
        let agent = crate::c2::Agent {
            identifier: format!("imix-v{}", imix_version),
        };

        let selectors = parse_host_unique_selectors();

        let host = crate::c2::Host {
            name: whoami::fallible::hostname().unwrap_or(String::from("")),
            identifier: host_unique::get_id_with_selectors(selectors).to_string(),
            platform: get_host_platform() as i32,
            primary_ip: get_primary_ip(),
        };

        let beacon_id =
            std::env::var("IMIX_BEACON_ID").unwrap_or_else(|_| String::from(Uuid::new_v4()));

        // Read callback URI at runtime (set by imix early in startup, or default).
        let rt = runtime_config();
        let transports = parse_transports(&rt.callback_uri);

        let available_transports = AvailableTransports {
            transports,
            active_index: 0,
        };

        let guardrails = parse_guardrails();
        if !guardrails::check_guardrails(guardrails) {
            #[cfg(feature = "print_debug")]
            log::error!("Guardrails failed, exiting");
            std::process::exit(0);
        }

        let info = crate::c2::Beacon {
            identifier: beacon_id,
            principal: whoami::username(),
            available_transports: Some(available_transports),
            host: Some(host),
            agent: Some(agent),
        };

        Config {
            info: Some(info),
            run_once: runtime_config().run_once,
        }
    }

    pub fn refresh_primary_ip(&mut self) {
        let fresh_ip = get_primary_ip();
        if self
            .info
            .clone()
            .is_some_and(|b| b.host.as_ref().is_some_and(|h| h.primary_ip != fresh_ip))
        {
            match self.info.clone() {
                Some(mut b) => match b.host.as_mut() {
                    Some(h) => {
                        h.primary_ip = fresh_ip;
                    }
                    None => {
                        #[cfg(feature = "print_debug")]
                        log::error!("host struct was never initialized, failed to set primary ip");
                    }
                },
                None => {
                    #[cfg(feature = "print_debug")]
                    log::error!("beacon struct was never initialized, failed to set primary ip");
                }
            }
        }
    }
}

fn get_host_platform() -> crate::c2::host::Platform {
    #[cfg(target_os = "linux")]
    return crate::c2::host::Platform::Linux;
    #[cfg(target_os = "macos")]
    return crate::c2::host::Platform::Macos;
    #[cfg(target_os = "windows")]
    return crate::c2::host::Platform::Windows;
    #[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
    return crate::c2::host::Platform::Bsd;
    #[cfg(all(
        not(target_os = "linux"),
        not(target_os = "macos"),
        not(target_os = "windows"),
        not(target_os = "freebsd"),
        not(target_os = "netbsd"),
        not(target_os = "openbsd"),
    ))]
    return crate::c2::host::Platform::Unspecified;
}

fn get_primary_ip() -> String {
    match netdev::get_default_interface() {
        Ok(default_interface) => match default_interface.ipv4.first() {
            Some(ip) => ip.addr().to_string(),
            None => String::from(""),
        },
        Err(_err) => {
            #[cfg(feature = "print_debug")]
            log::error!("failed to get primary ip: {_err}");
            String::from("")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_INTERVAL_SECONDS: u64 = 5;

    fn default_rt() -> RuntimeImixConfig {
        RuntimeImixConfig::default()
    }

    #[test]
    fn test_single_uri_parsing() {
        let rt = default_rt();
        let config = {
            // Construct Config equivalent manually for test without global init side-effects
            let transports = parse_transports(&rt.callback_uri);
            assert_eq!(transports.len(), 1);
            let expected_uri = rt.callback_uri.split(';').next().unwrap();
            let parsed_expected = Url::parse(expected_uri).unwrap();
            let mut expected_base = parsed_expected.clone();
            expected_base.set_query(None);
            assert!(transports[0].uri.starts_with(&expected_base.to_string()));
        };
        let _ = config;
    }

    #[test]
    fn test_transport_type_detection_grpc() {
        let grpc_type = get_transport_type("http://example.com");
        assert_eq!(grpc_type, crate::c2::transport::Type::TransportGrpc);
        let grpcs_type = get_transport_type("https://example.com");
        assert_eq!(grpcs_type, crate::c2::transport::Type::TransportGrpc);
    }

    #[test]
    fn test_transport_type_detection_http1() {
        let http1_type = get_transport_type("http1://example.com");
        assert_eq!(http1_type, crate::c2::transport::Type::TransportHttp1);
        let https1_type = get_transport_type("https1://example.com");
        assert_eq!(https1_type, crate::c2::transport::Type::TransportHttp1);
    }

    #[test]
    fn test_transport_type_detection_dns() {
        let dns_type = get_transport_type("dns://8.8.8.8");
        assert_eq!(dns_type, crate::c2::transport::Type::TransportDns);
    }

    #[test]
    fn test_transport_type_detection_unspecified() {
        let unknown_type = get_transport_type("ftp://example.com");
        assert_eq!(
            unknown_type,
            crate::c2::transport::Type::TransportUnspecified
        );
    }

    #[test]
    fn test_parse_callback_interval_valid() {
        let interval = parse_callback_interval().expect("Failed to parse callback interval");
        assert!(interval >= DEFAULT_INTERVAL_SECONDS);
    }

    #[test]
    fn test_config_creates_available_transports() {
        let config = Config::default_with_imix_version("v2");
        assert!(config.info.is_some());
        let info = config.info.unwrap();
        assert!(info.available_transports.is_some());
        let available = info.available_transports.unwrap();
        assert!(
            !available.transports.is_empty(),
            "Should have at least one transport"
        );
        assert_eq!(available.active_index, 0, "Active index should be 0");
    }

    #[test]
    fn test_empty_uri_filtered() {
        let uris = "http://example.com;;https://example2.com";
        let transports = parse_transports(uris);
        assert_eq!(transports.len(), 2);
        assert_eq!(transports[0].uri, "http://example.com/");
        assert_eq!(transports[1].uri, "https://example2.com/");
    }

    #[test]
    fn test_dsn_with_interval_query_param() {
        let uris = "https://example.com?interval=10";
        let transports = parse_transports(uris);
        assert_eq!(transports.len(), 1);
        assert_eq!(transports[0].uri, "https://example.com/");
        assert_eq!(transports[0].interval, 10);
        assert_eq!(transports[0].extra, "");
    }

    #[test]
    fn test_dsn_with_extra_query_param() {
        let uris = "https://example.com?extra=%7B%22key%22%3A%22value%22%7D";
        let transports = parse_transports(uris);
        assert_eq!(transports.len(), 1);
        assert_eq!(transports[0].uri, "https://example.com/");
        assert_eq!(transports[0].interval, DEFAULT_INTERVAL_SECONDS);
        assert_eq!(transports[0].extra, r#"{"key":"value"}"#);
    }

    #[test]
    fn test_dsn_with_both_query_params() {
        let uris = "https://example.com?interval=15&extra=%7B%22proxy%22%3A%22http%3A%2F%2Fproxy.local%22%7D";
        let transports = parse_transports(uris);
        assert_eq!(transports.len(), 1);
        assert_eq!(transports[0].uri, "https://example.com/");
        assert_eq!(transports[0].interval, 15);
        assert_eq!(transports[0].extra, r#"{"proxy":"http://proxy.local"}"#);
    }

    #[test]
    fn test_dsn_multiple_uris_with_different_params() {
        let uris = "https://primary.com?interval=10;https://fallback.com?interval=30";
        let transports = parse_transports(uris);
        assert_eq!(transports.len(), 2);
        assert_eq!(transports[0].uri, "https://primary.com/");
        assert_eq!(transports[0].interval, 10);
        assert_eq!(transports[1].uri, "https://fallback.com/");
        assert_eq!(transports[1].interval, 30);
    }

    #[test]
    fn test_dsn_no_query_params_uses_defaults() {
        let uris = "https://example.com";
        let transports = parse_transports(uris);
        assert_eq!(transports.len(), 1);
        assert_eq!(transports[0].uri, "https://example.com/");
        assert_eq!(transports[0].interval, DEFAULT_INTERVAL_SECONDS);
        let rt = default_rt();
        assert_eq!(transports[0].extra, rt.transport_extra.to_lowercase());
    }

    #[test]
    fn test_dsn_invalid_interval_uses_default() {
        let uris = "https://example.com?interval=invalid";
        let transports = parse_transports(uris);
        assert_eq!(transports.len(), 0);
    }

    #[test]
    fn test_dsn_mixed_with_and_without_params() {
        let uris = "https://first.com?interval=10;https://second.com;https://third.com?interval=25";
        let transports = parse_transports(uris);
        assert_eq!(transports.len(), 3);
        assert_eq!(transports[0].interval, 10);
        assert_eq!(transports[1].interval, DEFAULT_INTERVAL_SECONDS);
        assert_eq!(transports[2].interval, 25);
    }

    #[test]
    fn test_dsn_with_unencoded_json() {
        let uris =
            r#"https://example.com?interval=20&extra={"key":"value","nested":{"Foo":"Bar"}}"#;
        let transports = parse_transports(uris);
        assert_eq!(transports.len(), 1);
        assert_eq!(transports[0].uri, "https://example.com/");
        assert_eq!(transports[0].interval, 20);
        assert_eq!(
            transports[0].extra,
            r#"{"key":"value","nested":{"foo":"bar"}}"#
        );
    }

    #[test]
    fn test_dsn_with_jitter() {
        let uris = "https://example.com?jitter=0.5";
        let transports = parse_transports(uris);
        assert_eq!(transports.len(), 1);
        assert_eq!(transports[0].uri, "https://example.com/");
        assert_eq!(transports[0].jitter, 0.5);
    }

    #[test]
    fn test_transport_type_detection_quic() {
        let quic_type = get_transport_type("quic://example.com");
        assert_eq!(quic_type, crate::c2::transport::Type::TransportQuic);
        let quics_type = get_transport_type("quics://example.com");
        assert_eq!(quics_type, crate::c2::transport::Type::TransportQuic);
    }

    #[test]
    fn test_dsn_with_type_query_param() {
        let uris = "http://example.com?type=quic";
        let transports = parse_transports(uris);
        assert_eq!(transports.len(), 1);
        assert_eq!(transports[0].uri, "http://example.com/");
        assert_eq!(
            transports[0].r#type,
            crate::c2::transport::Type::TransportQuic as i32
        );
    }
}
