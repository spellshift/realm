use uuid::Uuid;
use crate::HostIDSelector;

#[derive(Default)]
pub struct Registry {
    subkey: Option<String>,
    value_name: Option<String>,
}

impl Registry {
    /*
     * Returns a predefined path to the host id registry key.
     */
    pub fn with_subkey(mut self, path: impl Into<String>) -> Self {
        self.subkey = Some(path.into());
        self
    }

    pub fn with_value_name(mut self, name: impl Into<String>) -> Self {
        self.value_name = Some(name.into());
        self
    }

    fn key_path(&self) -> &str {
        self.subkey
            .as_deref()
            .unwrap_or("SOFTWARE\\Imix")
    }

    fn val_name(&self) -> &str {
        self.value_name
            .as_deref()
            .unwrap_or("HostId")
    }
}

impl HostIDSelector for Registry {
    fn get_name(&self) -> String {
        "registry".into()
    }

    fn get_host_id(&self) -> Option<Uuid> {
        // On non‑Windows targets this selector is unavailable
        #[cfg(not(target_os = "windows"))]
        {
            return None;
        }

        #[cfg(target_os = "windows")]
        {
            use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

            // Open or create our key under HKLM
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let (key, _) = match hklm.create_subkey(self.key_path()) {
                Ok(pair) => pair,
                Err(err) => {
                    #[cfg(debug_assertions)]
                    log::debug!("could not open/create registry key: {:?}", err);
                    return None;
                }
            };

            // Try to read existing value
            if let Ok(stored) = key.get_value::<String, _>(self.val_name()) {
                if let Ok(uuid) = Uuid::parse_str(&stored) {
                    return Some(uuid);
                } else {
                    #[cfg(debug_assertions)]
                    log::debug!("invalid UUID in registry: {:?}", stored);
                }
            }

            // Otherwise generate a new one and persist it
            let new_uuid = Uuid::new_v4();
            let s = new_uuid.to_string();
            if let Err(err) = key.set_value(self.val_name(), &s) {
                #[cfg(debug_assertions)]
                log::debug!("failed to write registry value: {:?}", err);
            }
            Some(new_uuid)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winreg::RegKey;
    use winreg::enums::HKEY_LOCAL_MACHINE;

    #[test]
    fn test_registry() {
        let selector = Registry::default();
        let id_one = selector.get_host_id();
        let id_two = selector.get_host_id();

        assert!(id_one.is_some());
        assert!(id_two.is_some());
        assert_eq!(id_one, id_two);

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let _ = hklm.delete_subkey_all(selector.key_path());
    }
}