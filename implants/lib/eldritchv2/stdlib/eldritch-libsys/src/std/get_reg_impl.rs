use alloc::collections::BTreeMap;
use alloc::string::String;
use anyhow::Result;

#[cfg(not(target_os = "windows"))]
pub fn get_reg(_reghive: String, _regpath: String) -> Result<BTreeMap<String, String>> {
    Err(anyhow::anyhow!(
        "This OS isn't supported by the get_reg function. Only windows systems are supported"
    ))
}

#[cfg(target_os = "windows")]
pub fn get_reg(reghive: String, regpath: String) -> Result<BTreeMap<String, String>> {
    let mut tmp_res = BTreeMap::new();

    use winreg::{enums::*, RegKey, RegValue};
    //Accepted values for reghive :
    //HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, HKEY_USERS, HKEY_PERFORMANCE_DATA, HKEY_PERFORMANCE_TEXT, HKEY_PERFORMANCE_NLSTEXT, HKEY_CURRENT_CONFIG, HKEY_DYN_DATA, HKEY_CURRENT_USER_LOCAL_SETTINGS

    let ihive: isize = match reghive.as_ref() {
        "HKEY_CLASSES_ROOT" => HKEY_CLASSES_ROOT,
        "HKEY_CURRENT_USER" => HKEY_CURRENT_USER,
        "HKEY_LOCAL_MACHINE" => HKEY_LOCAL_MACHINE,
        "HKEY_USERS" => HKEY_USERS,
        "HKEY_PERFORMANCE_DATA" => HKEY_PERFORMANCE_DATA,
        "HKEY_PERFORMANCE_TEXT" => HKEY_PERFORMANCE_TEXT,
        "HKEY_PERFORMANCE_NLSTEXT" => HKEY_PERFORMANCE_NLSTEXT,
        "HKEY_CURRENT_CONFIG" => HKEY_CURRENT_CONFIG,
        "HKEY_DYN_DATA" => HKEY_DYN_DATA,
        "HKEY_CURRENT_USER_LOCAL_SETTINGS" => HKEY_CURRENT_USER_LOCAL_SETTINGS,
        _ => return Err(anyhow::anyhow!("RegHive can only be one of the following values - HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, HKEY_USERS, HKEY_PERFORMANCE_DATA, HKEY_PERFORMANCE_TEXT, HKEY_PERFORMANCE_NLSTEXT, HKEY_CURRENT_CONFIG, HKEY_DYN_DATA, HKEY_CURRENT_USER_LOCAL_SETTINGS ")),

    };

    let hive = RegKey::predef(ihive);
    let subkey = hive.open_subkey(regpath)?;

    for result in subkey.enum_values() {
        let (key, val): (String, RegValue) = result?;
        tmp_res.insert(key, val.to_string());
    }
    Ok(tmp_res)
}

#[cfg(target_os = "windows")]
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use winreg::{enums::*, RegKey};

    #[test]
    fn test_get_reg() -> anyhow::Result<()> {
        let id = Uuid::new_v4();
        //Write something into temp regkey...
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (nkey, _ndisp) = hkcu.create_subkey(format!("SOFTWARE\\{}", id))?;
        nkey.set_value("FOO", &"BAR")?;

        let ares = get_reg("HKEY_CURRENT_USER".to_string(), format!("SOFTWARE\\{}", id))?;
        let val2 = ares.get("FOO").unwrap();
        //delete temp regkey
        hkcu.delete_subkey(format!("SOFTWARE\\{}", id))?;

        assert_eq!(val2, "BAR");

        Ok(())
    }
}
