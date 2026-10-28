use anyhow::Result;
use winreg::enums::*;

pub fn parse_registry_path(path: &str) -> Result<(isize, &str)> {
    let mut parts = path.splitn(2, '\\');
    let hive_str = parts.next().unwrap_or("");
    let subkey = parts.next().unwrap_or("");

    let ihive = match hive_str {
        "HKEY_CLASSES_ROOT" | "HKCR" => HKEY_CLASSES_ROOT,
        "HKEY_CURRENT_USER" | "HKCU" => HKEY_CURRENT_USER,
        "HKEY_LOCAL_MACHINE" | "HKLM" => HKEY_LOCAL_MACHINE,
        "HKEY_USERS" | "HKU" => HKEY_USERS,
        "HKEY_PERFORMANCE_DATA" | "HKPD" => HKEY_PERFORMANCE_DATA,
        "HKEY_PERFORMANCE_TEXT" | "HKPT" => HKEY_PERFORMANCE_TEXT,
        "HKEY_PERFORMANCE_NLSTEXT" | "HKPN" => HKEY_PERFORMANCE_NLSTEXT,
        "HKEY_CURRENT_CONFIG" | "HKCC" => HKEY_CURRENT_CONFIG,
        "HKEY_DYN_DATA" | "HKDD" => HKEY_DYN_DATA,
        "HKEY_CURRENT_USER_LOCAL_SETTINGS" | "HKCULS" => HKEY_CURRENT_USER_LOCAL_SETTINGS,
        _ => {
            return Err(anyhow::anyhow!(
                "RegistryHive can only be one of the following values - HKEY_CLASSES_ROOT (HKCR), HKEY_CURRENT_USER (HKCU), HKEY_LOCAL_MACHINE (HKLM), HKEY_USERS (HKU), HKEY_PERFORMANCE_DATA (HKPD), HKEY_PERFORMANCE_TEXT (HKPT), HKEY_PERFORMANCE_NLSTEXT (HKPN), HKEY_CURRENT_CONFIG (HKCC), HKEY_DYN_DATA (HKDD), HKEY_CURRENT_USER_LOCAL_SETTINGS (HKCULS) "
            ));
        }
    };

    Ok((ihive, subkey))
}
