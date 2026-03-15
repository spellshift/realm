use serde::{Deserialize, Serialize};

use crate::Guardrail;

#[derive(Serialize, Deserialize, Default)]
pub struct Registry {
    #[serde(default)]
    pub subkey: String,
    #[serde(default)]
    pub value_name: Option<String>,
}

impl Registry {
    pub fn new(subkey: &str, value_name: Option<String>) -> Self {
        Registry {
            subkey: subkey.to_string(),
            value_name,
        }
    }
}

impl Guardrail for Registry {
    fn get_name(&self) -> String {
        "registry".to_string()
    }

    #[cfg(target_os = "windows")]
    fn check(&self) -> bool {
        use winreg::enums::*;
        use winreg::RegKey;

        if self.subkey.is_empty() {
            return false;
        }

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

        let check_key = |root_key: &RegKey| -> bool {
            if let Ok(key) = root_key.open_subkey(&self.subkey) {
                if let Some(val_name) = &self.value_name {
                    // Try to read any value type to check if it exists
                    // We just need to know if the value name exists in the key
                    return key.enum_values().any(|val| {
                        if let Ok((name, _)) = val {
                            name == *val_name
                        } else {
                            false
                        }
                    });
                }
                return true;
            }
            false
        };

        check_key(&hkcu) || check_key(&hklm)
    }

    #[cfg(not(target_os = "windows"))]
    fn check(&self) -> bool {
        false
    }
}

#[cfg(test)]
#[cfg(target_os = "windows")]
mod tests {
    use super::*;

    #[test]
    fn test_registry_guardrail_exists() {
        let guardrail = Registry::new("SOFTWARE\\Microsoft\\Windows\\CurrentVersion", None);
        assert!(guardrail.check());
    }

    #[test]
    fn test_registry_guardrail_not_exists() {
        let guardrail = Registry::new("SOFTWARE\\NonExistentKey12345", None);
        assert!(!guardrail.check());
    }

    #[test]
    fn test_registry_value_exists() {
        let guardrail = Registry::new(
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion",
            Some("ProgramFilesDir".to_string()),
        );
        assert!(guardrail.check());
    }

    #[test]
    fn test_registry_value_not_exists() {
        let guardrail = Registry::new(
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion",
            Some("NonExistentValue12345".to_string()),
        );
        assert!(!guardrail.check());
    }
}
