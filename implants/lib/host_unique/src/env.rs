use std::env;
use uuid::Uuid;

use crate::HostIDSelector;

#[derive(Default)]
pub struct Env {}

impl HostIDSelector for Env {
    fn get_name(&self) -> String {
        "env".to_string()
    }

    fn get_host_id(&self) -> Option<uuid::Uuid> {
        let host_id_env = match env::var("IMIX_HOST_ID") {
            Ok(res) => res,
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::debug!("No environment variable set {:?}", _err);
                return None;
            }
        };
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

#[cfg(test)]
mod tests {
    use uuid::uuid;

    use super::*;

    #[test]
    fn test_id_env() {
        std::env::set_var("IMIX_HOST_ID", "f17b92c0-e383-4328-9017-952e5d9fd53d");
        let selector = Env {};
        let id = selector.get_host_id().unwrap();

        assert_eq!(id, uuid!("f17b92c0-e383-4328-9017-952e5d9fd53d"));
    }
}
