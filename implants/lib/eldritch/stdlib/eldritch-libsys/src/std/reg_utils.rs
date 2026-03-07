use alloc::string::{String, ToString};
use anyhow::Result;

#[cfg(target_os = "windows")]
use winreg::enums::*;

#[cfg(target_os = "windows")]
pub fn parse_registry_path(path: &str) -> Result<(isize, String)> {
    let mut parts = path.splitn(2, '\\');
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
