use crate::constants::proxy_uri;

pub fn get_system_proxy() -> Option<String> {
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
pub fn get_host_platform() -> pb::c2::host::Platform {
    #[cfg(target_os = "linux")]
    return pb::c2::host::Platform::Linux;

    #[cfg(target_os = "macos")]
    return pb::c2::host::Platform::Macos;

    #[cfg(target_os = "windows")]
    return pb::c2::host::Platform::Windows;

    #[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
    return pb::c2::host::Platform::Bsd;

    #[cfg(all(
        not(target_os = "linux"),
        not(target_os = "macos"),
        not(target_os = "windows"),
        not(target_os = "freebsd"),
        not(target_os = "netbsd"),
        not(target_os = "openbsd"),
    ))]
    return pb::c2::host::Platform::Unspecified;
}

/*
 * Return the first IPv4 address of the default interface as a string.
 * Returns the empty string otherwise.
 */
pub fn get_primary_ip() -> String {
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
