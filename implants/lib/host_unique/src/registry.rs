use crate::HostIDSelector;
use uuid::Uuid;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Registry {
    #[serde(default)]
    subkey: Option<String>,
    #[serde(default)]
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

    #[cfg(target_os = "windows")]
    fn key_path(&self) -> &str {
        self.subkey.as_deref().unwrap_or("SOFTWARE\\Imix")
    }

    #[cfg(target_os = "windows")]
    fn val_name(&self) -> &str {
        self.value_name.as_deref().unwrap_or("system-id")
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
            None
        }

        #[cfg(target_os = "windows")]
        {
            use std::io::ErrorKind;
            use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

            // Try to open the key for reading
            let key = match hklm.open_subkey(self.key_path()) {
                Ok(k) => k,
                Err(_err) if _err.kind() == ErrorKind::NotFound => {
                    // If it doesn't exist, create it
                    match hklm.create_subkey(self.key_path()) {
                        Ok((k, _disp)) => k,
                        Err(_err) => {
                            #[cfg(debug_assertions)]
                            log::debug!("failed to create registry key: {:?}", _err);
                            return None;
                        }
                    }
                }
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::debug!("failed to open registry key: {:?}", _err);
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
            if let Err(_err) = key.set_value(self.val_name(), &s) {
                #[cfg(debug_assertions)]
                log::debug!("failed to write registry value: {:?}", _err);
            }
            Some(new_uuid)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "windows")]
    #[test]
    fn test_registry() {
        use winreg::enums::HKEY_LOCAL_MACHINE;
        use winreg::RegKey;

        let selector = Registry::default();
        let id_one = selector.get_host_id();
        let id_two = selector.get_host_id();

        assert!(id_one.is_some());
        assert!(id_two.is_some());
        assert_eq!(id_one, id_two);

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let _ = hklm.delete_subkey_all(selector.key_path());
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_registry_non_windows() {
        let selector = Registry::default();
        // on non‑Windows we expect registry lookup to be None
        assert!(selector.get_host_id().is_none());
    }
}
