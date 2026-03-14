use alloc::collections::BTreeMap;
use alloc::string::String;
use anyhow::Result;

#[cfg(target_os = "windows")]
use crate::std::reg_utils::parse_registry_path;

#[cfg(not(target_os = "windows"))]
pub fn get_reg(_path: String) -> Result<BTreeMap<String, String>> {
    Err(anyhow::anyhow!(
        "This OS isn't supported by the get_reg function. Only windows systems are supported"
    ))
}

#[cfg(target_os = "windows")]
pub fn get_reg(path: String) -> Result<BTreeMap<String, String>> {
    let mut tmp_res = BTreeMap::new();

    use winreg::{RegKey, RegValue, enums::*};

    let (ihive, subkey_str) = parse_registry_path(&path)?;

    let hive = RegKey::predef(ihive);
    let subkey = hive.open_subkey(subkey_str)?;

    for result in subkey.enum_values() {
        let (key, val): (String, RegValue) = result?;
        tmp_res.insert(key, val.to_string());
    }
    Ok(tmp_res)
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "windows")]
    use super::*;
    #[cfg(target_os = "windows")]
    use uuid::Uuid;
    #[cfg(target_os = "windows")]
    use winreg::{RegKey, enums::*};

    #[test]
    #[cfg(target_os = "windows")]
    fn test_get_reg() -> anyhow::Result<()> {
        let id = Uuid::new_v4();
        //Write something into temp regkey...
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (nkey, _ndisp) = hkcu.create_subkey(format!("SOFTWARE\\{}", id))?;
        nkey.set_value("FOO", &"BAR")?;

        let ares = get_reg(format!("HKCU\\SOFTWARE\\{}", id))?;
        let val2 = ares.get("FOO").unwrap();
        //delete temp regkey
        hkcu.delete_subkey(format!("SOFTWARE\\{}", id))?;

        assert_eq!(val2, "BAR");

        Ok(())
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_get_reg_non_windows() {
        let res = super::get_reg("HKCU\\SOFTWARE".into());
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .to_string()
                .contains("Only windows systems are supported")
        );
    }
}
