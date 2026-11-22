use serde::{Deserialize, Serialize};

use crate::Guardrail;

#[derive(Serialize, Deserialize, Default)]
pub struct Registry {
    #[serde(default, alias = "subkey")]
    pub path: String,
    #[serde(default)]
    pub value_name: Option<String>,
}

#[cfg(any(target_os = "windows", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegistryHive {
    ClassesRoot,
    CurrentUser,
    LocalMachine,
    Users,
    PerformanceData,
    PerformanceText,
    PerformanceNlsText,
    CurrentConfig,
    DynData,
    CurrentUserLocalSettings,
}

#[cfg(any(target_os = "windows", test))]
fn parse_hive(hive: &str) -> Option<RegistryHive> {
    const HIVES: &[(&str, RegistryHive)] = &[
        ("HKEY_CLASSES_ROOT", RegistryHive::ClassesRoot),
        ("HKCR", RegistryHive::ClassesRoot),
        ("HKEY_CURRENT_USER", RegistryHive::CurrentUser),
        ("HKCU", RegistryHive::CurrentUser),
        ("HKEY_LOCAL_MACHINE", RegistryHive::LocalMachine),
        ("HKLM", RegistryHive::LocalMachine),
        ("HKEY_USERS", RegistryHive::Users),
        ("HKU", RegistryHive::Users),
        ("HKEY_PERFORMANCE_DATA", RegistryHive::PerformanceData),
        ("HKPD", RegistryHive::PerformanceData),
        ("HKEY_PERFORMANCE_TEXT", RegistryHive::PerformanceText),
        ("HKPT", RegistryHive::PerformanceText),
        ("HKEY_PERFORMANCE_NLSTEXT", RegistryHive::PerformanceNlsText),
        ("HKPN", RegistryHive::PerformanceNlsText),
        ("HKEY_CURRENT_CONFIG", RegistryHive::CurrentConfig),
        ("HKCC", RegistryHive::CurrentConfig),
        ("HKEY_DYN_DATA", RegistryHive::DynData),
        ("HKDD", RegistryHive::DynData),
        (
            "HKEY_CURRENT_USER_LOCAL_SETTINGS",
            RegistryHive::CurrentUserLocalSettings,
        ),
        ("HKCULS", RegistryHive::CurrentUserLocalSettings),
    ];

    HIVES
        .iter()
        .find(|(name, _)| hive.eq_ignore_ascii_case(name))
        .map(|(_, h)| *h)
}

#[cfg(any(target_os = "windows", test))]
fn parse_registry_path(path: &str) -> Option<(RegistryHive, &str)> {
    let (hive_str, subkey) = path.split_once('\\')?;
    let subkey = subkey.trim_start_matches('\\');
    if subkey.is_empty() {
        return None;
    }
    let hive = parse_hive(hive_str)?;
    Some((hive, subkey))
}

#[cfg(target_os = "windows")]
impl RegistryHive {
    fn open_predef(self) -> winreg::RegKey {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkey = match self {
            RegistryHive::ClassesRoot => HKEY_CLASSES_ROOT,
            RegistryHive::CurrentUser => HKEY_CURRENT_USER,
            RegistryHive::LocalMachine => HKEY_LOCAL_MACHINE,
            RegistryHive::Users => HKEY_USERS,
            RegistryHive::PerformanceData => HKEY_PERFORMANCE_DATA,
            RegistryHive::PerformanceText => HKEY_PERFORMANCE_TEXT,
            RegistryHive::PerformanceNlsText => HKEY_PERFORMANCE_NLSTEXT,
            RegistryHive::CurrentConfig => HKEY_CURRENT_CONFIG,
            RegistryHive::DynData => HKEY_DYN_DATA,
            RegistryHive::CurrentUserLocalSettings => HKEY_CURRENT_USER_LOCAL_SETTINGS,
        };
        RegKey::predef(hkey)
    }
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
        let Some((hive, subkey)) = parse_registry_path(&self.path) else {
            return false;
        };

        let root_key = hive.open_predef();

        let Ok(key) = root_key.open_subkey(subkey) else {
            return false;
        };

        if let Some(val_name) = &self.value_name {
            return key.get_raw_value(val_name).is_ok();
        }

        true
    }

    #[cfg(not(target_os = "windows"))]
    fn check(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_path_field() {
        let json = r#"{"path": "HKLM\\SOFTWARE\\Test", "value_name": "Version"}"#;
        let guardrail: Registry = serde_json::from_str(json).expect("deserialize failed");
        assert_eq!(guardrail.path, "HKLM\\SOFTWARE\\Test");
        assert_eq!(guardrail.value_name.as_deref(), Some("Version"));
    }

    #[test]
    fn test_deserialize_subkey_alias() {
        let json = r#"{"subkey": "HKLM\\SOFTWARE\\Test", "value_name": null}"#;
        let guardrail: Registry = serde_json::from_str(json).expect("deserialize failed");
        assert_eq!(guardrail.path, "HKLM\\SOFTWARE\\Test");
        assert_eq!(guardrail.value_name, None);
    }

    #[test]
    fn test_parse_registry_path_hives_and_aliases() {
        let cases = [
            ("HKEY_CLASSES_ROOT\\foo", RegistryHive::ClassesRoot),
            ("hkcr\\foo", RegistryHive::ClassesRoot),
            ("HKEY_CURRENT_USER\\foo", RegistryHive::CurrentUser),
            ("HKCU\\foo", RegistryHive::CurrentUser),
            ("HKEY_LOCAL_MACHINE\\foo", RegistryHive::LocalMachine),
            ("hKlM\\foo", RegistryHive::LocalMachine),
            ("HKEY_USERS\\foo", RegistryHive::Users),
            ("HKU\\foo", RegistryHive::Users),
            ("HKEY_PERFORMANCE_DATA\\foo", RegistryHive::PerformanceData),
            ("HKPD\\foo", RegistryHive::PerformanceData),
            ("HKEY_PERFORMANCE_TEXT\\foo", RegistryHive::PerformanceText),
            ("HKPT\\foo", RegistryHive::PerformanceText),
            (
                "HKEY_PERFORMANCE_NLSTEXT\\foo",
                RegistryHive::PerformanceNlsText,
            ),
            ("HKPN\\foo", RegistryHive::PerformanceNlsText),
            ("HKEY_CURRENT_CONFIG\\foo", RegistryHive::CurrentConfig),
            ("HKCC\\foo", RegistryHive::CurrentConfig),
            ("HKEY_DYN_DATA\\foo", RegistryHive::DynData),
            ("HKDD\\foo", RegistryHive::DynData),
            (
                "HKEY_CURRENT_USER_LOCAL_SETTINGS\\foo",
                RegistryHive::CurrentUserLocalSettings,
            ),
            ("hkculs\\foo", RegistryHive::CurrentUserLocalSettings),
        ];

        for (input, expected_hive) in cases {
            assert_eq!(
                parse_registry_path(input),
                Some((expected_hive, "foo")),
                "failed for input {input}"
            );
        }
    }

    #[test]
    fn test_parse_registry_path_rejects_empty_subkey_and_invalid_hive() {
        assert_eq!(parse_registry_path(""), None);
        assert_eq!(parse_registry_path("HKLM"), None);
        assert_eq!(parse_registry_path("HKLM\\"), None);
        assert_eq!(parse_registry_path("HKLM\\\\"), None);
        assert_eq!(parse_registry_path("\\SOFTWARE\\Test"), None);
        assert_eq!(parse_registry_path("INVALID\\SOFTWARE\\Test"), None);
    }

    #[test]
    fn test_parse_registry_path_trims_leading_subkey_backslashes() {
        assert_eq!(
            parse_registry_path("HKLM\\\\SOFTWARE\\Microsoft"),
            Some((RegistryHive::LocalMachine, "SOFTWARE\\Microsoft"))
        );
    }
}

#[cfg(test)]
#[cfg(target_os = "windows")]
mod windows_tests {
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
    fn test_registry_hive_only_rejected() {
        assert!(!Registry::new("HKLM", None).check());
        assert!(!Registry::new("HKLM\\", None).check());
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
    fn test_registry_value_exists_case_insensitive() {
        let guardrail = Registry::new(
            "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion",
            Some("programfilesdir".to_string()),
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
