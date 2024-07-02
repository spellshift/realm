use std::env;
use uuid::Uuid;

use crate::HostUniqueEngine;

pub struct Env {}

impl HostUniqueEngine for Env {
    fn get_name(&self) -> String {
        "env".to_string()
    }

    fn get_host_id(&self) -> Option<uuid::Uuid> {
        let host_id_env = env::var("IMIX_HOST_ID").unwrap();
        match Uuid::parse_str(&host_id_env) {
            Ok(res) => Some(res),
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::debug!("Failed to deploy {:?}", _err);
                None
            }
        }
    }
}
