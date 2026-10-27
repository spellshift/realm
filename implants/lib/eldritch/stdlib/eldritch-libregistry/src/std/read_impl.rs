use alloc::collections::BTreeMap;
use alloc::string::String;
use anyhow::Result;

#[allow(unused_variables)]
pub fn read_reg(path: String) -> Result<BTreeMap<String, String>> {
    #[cfg(not(target_os = "windows"))]
    return Err(anyhow::anyhow!(
        "This OS isn't supported by the registry.read function. Only windows systems are supported"
    ));

    #[cfg(target_os = "windows")]
    {
        use super::reg_utils::parse_registry_path;
        use alloc::format;
        use winreg::{RegKey, RegValue};

        let mut res = BTreeMap::new();

        let (ihive, subkey_path) = parse_registry_path(&path)?;

        let hive = RegKey::predef(ihive);
        let subkey = hive.open_subkey(subkey_path)?;

        for val in subkey.enum_values() {
            let (name, val): (String, RegValue) = val?;
            res.insert(name, format!("{}", val));
        }

        Ok(res)
    }
}

#[cfg(test)]
#[cfg(target_os = "windows")]
mod tests {
    use super::*;
    use alloc::format;
    use uuid::Uuid;
    use winreg::{RegKey, enums::*};

    #[test]
    fn test_read_reg() -> anyhow::Result<()> {
        let id = Uuid::new_v4();

        // -------------------- WRITE INTO TEMP REGKEY...
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _disp) = hkcu.create_subkey(format!("SOFTWARE\\Temp\\{}", id))?;
        key.set_value("FOO", &"BAR")?;

        // --------------------- READ FROM TEMP REGKEY...
        let tmp_res = read_reg(format!("HKEY_CURRENT_USER\\SOFTWARE\\Temp\\{}", id))?;
        assert_eq!(tmp_res.get("FOO").unwrap(), "BAR");

        // --------------------- DELETE TEMP REGKEY...
        hkcu.delete_subkey(format!("SOFTWARE\\Temp\\{}", id))?;

        Ok(())
    }
}
