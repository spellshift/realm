use alloc::string::{String, ToString};
use anyhow::Result;

#[cfg(target_os = "windows")]
use winreg::enums::*;

#[cfg(target_os = "windows")]
pub fn parse_registry_path(path: &str) -> Result<(isize, String)> {
    // Normalize double backslashes to single backslashes
    let normalized_path = path.replace("\\\\", "\\");

    let mut parts = normalized_path.splitn(2, '\\');
    let hive_str = parts.next().unwrap_or("");
    let subkey_str = parts.next().unwrap_or("").to_string();

    let ihive: isize = match hive_str {
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
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid RegHive: {}. Supported values include HKEY_LOCAL_MACHINE, HKLM, HKEY_CURRENT_USER, HKCU, etc.",
                hive_str
            ));
        }
    };

    Ok((ihive, subkey_str))
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "windows")]
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_parse_registry_path_single_backslash() {
        let (hive, subkey) = parse_registry_path("HKLM\\SOFTWARE\\Microsoft").unwrap();
        assert_eq!(hive, HKEY_LOCAL_MACHINE);
        assert_eq!(subkey, "SOFTWARE\\Microsoft");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_parse_registry_path_double_backslash() {
        let (hive, subkey) = parse_registry_path("HKLM\\\\SOFTWARE\\\\Microsoft").unwrap();
        assert_eq!(hive, HKEY_LOCAL_MACHINE);
        assert_eq!(subkey, "SOFTWARE\\Microsoft");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_parse_registry_path_mixed_backslash() {
        let (hive, subkey) = parse_registry_path("HKEY_CURRENT_USER\\\\SOFTWARE\\Microsoft").unwrap();
        assert_eq!(hive, HKEY_CURRENT_USER);
        assert_eq!(subkey, "SOFTWARE\\Microsoft");
    }
}
