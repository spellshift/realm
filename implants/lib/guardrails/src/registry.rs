use serde::{Deserialize, Serialize};

use crate::Guardrail;

#[derive(Serialize, Deserialize, Default)]
pub struct Registry {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub value_name: Option<String>,
}

impl Registry {
    pub fn new(path: &str, value_name: Option<String>) -> Self {
        Registry {
            path: path.to_string(),
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

        if self.path.is_empty() {
            return false;
        }

        let normalized_path = self.path.replace("\\\\", "\\");
        let mut parts = normalized_path.splitn(2, '\\');
        let hive_str = parts.next().unwrap_or("");
        let subkey_str = parts.next().unwrap_or("").to_string();

        let ihive = match hive_str.to_ascii_uppercase().as_str() {
            "HKEY_CLASSES_ROOT" | "HKCR" => HKEY_CLASSES_ROOT,
            "HKEY_CURRENT_USER" | "HKCU" => HKEY_CURRENT_USER,
            "HKEY_LOCAL_MACHINE" | "HKLM" => HKEY_LOCAL_MACHINE,
            "HKEY_USERS" | "HKU" => HKEY_USERS,
            "HKEY_PERFORMANCE_DATA" => HKEY_PERFORMANCE_DATA,
            "HKEY_PERFORMANCE_TEXT" => HKEY_PERFORMANCE_TEXT,
            "HKEY_PERFORMANCE_NLSTEXT" => HKEY_PERFORMANCE_NLSTEXT,
            "HKEY_CURRENT_CONFIG" | "HKCC" => HKEY_CURRENT_CONFIG,
            "HKEY_DYN_DATA" => HKEY_DYN_DATA,
            "HKEY_CURRENT_USER_LOCAL_SETTINGS" => HKEY_CURRENT_USER_LOCAL_SETTINGS,
            _ => return false,
        };

        let root_key = RegKey::predef(ihive);

        if let Ok(key) = root_key.open_subkey(&subkey_str) {
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
        let guardrail = Registry::new("HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion", None);
        assert!(guardrail.check());
    }

    #[test]
    fn test_registry_guardrail_not_exists() {
        let guardrail = Registry::new("HKLM\\SOFTWARE\\NonExistentKey12345", None);
        assert!(!guardrail.check());
    }

    #[test]
    fn test_registry_value_exists() {
        let guardrail = Registry::new(
            "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion",
            Some("ProgramFilesDir".to_string()),
        );
        assert!(guardrail.check());
    }

    #[test]
    fn test_registry_value_not_exists() {
        let guardrail = Registry::new(
            "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion",
            Some("NonExistentValue12345".to_string()),
        );
        assert!(!guardrail.check());
    }
}
