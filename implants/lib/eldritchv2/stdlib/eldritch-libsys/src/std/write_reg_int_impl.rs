use anyhow::Result;
use alloc::string::String;

#[allow(unused_variables)]
pub fn write_reg_int(
    reghive: String,
    regpath: String,
    regname: String,
    regtype: String,
    regvalue: u32,
) -> Result<bool> {
    #[cfg(not(target_os = "windows"))]
    return Err(anyhow::anyhow!(
        "This OS isn't supported by the write_reg function. Only windows systems are supported"
    ));

    #[cfg(target_os = "windows")]
    {
        use winreg::{enums::*, RegKey, RegValue};

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
        let (nkey, _ndisp) = hive.create_subkey(regpath)?;

        match regtype.as_ref() {
            "REG_NONE" => {
                nkey.set_value(regname, &regvalue)?;
            },
            "REG_SZ" => nkey.set_value(regname, &regvalue)?,
            "REG_EXPAND_SZ" => nkey.set_value(regname, &regvalue)?,
            "REG_BINARY" => {
                let data = RegValue{ vtype: REG_BINARY, bytes: regvalue.to_le_bytes().to_vec()};
                nkey.set_raw_value(regname, &data)?;
            },
            "REG_DWORD" => {
                let data = RegValue{ vtype: REG_DWORD, bytes: regvalue.to_le_bytes().to_vec()};
                nkey.set_raw_value(regname, &data)?;
            },
            "REG_DWORD_BIG_ENDIAN" => {
                let data = RegValue{ vtype: REG_DWORD_BIG_ENDIAN, bytes: regvalue.to_be_bytes().to_vec()};
                nkey.set_raw_value(regname, &data)?;
            },
            "REG_LINK" => {
                nkey.set_value(regname, &regvalue)?;
            },
            "REG_MULTI_SZ" => {
                nkey.set_value(regname, &regvalue)?;
            },
            "REG_RESOURCE_LIST" => {
                let data = RegValue{ vtype: REG_RESOURCE_LIST, bytes: regvalue.to_le_bytes().to_vec()};
                nkey.set_raw_value(regname, &data)?;
            },
            "REG_FULL_RESOURCE_DESCRIPTOR" => {
                let data = RegValue{ vtype: REG_FULL_RESOURCE_DESCRIPTOR, bytes: regvalue.to_le_bytes().to_vec()};
                nkey.set_raw_value(regname, &data)?;
            },
            "REG_RESOURCE_REQUIREMENTS_LIST" => {
                let data = RegValue{ vtype: REG_RESOURCE_REQUIREMENTS_LIST, bytes: regvalue.to_le_bytes().to_vec()};
                nkey.set_raw_value(regname, &data)?;
            },
            "REG_QWORD" => {
                let data = RegValue{ vtype: REG_QWORD, bytes: (regvalue as u64).to_le_bytes().to_vec()};
                nkey.set_raw_value(regname, &data)?;
            },
            _ => return Err(anyhow::anyhow!("RegType can only be one of the following values - REG_NONE, REG_SZ, REG_EXPAND_SZ, REG_BINARY, REG_DWORD, REG_DWORD_BIG_ENDIAN, REG_LINK, REG_MULTI_SZ, REG_RESOURCE_LIST, REG_RESOURCE_LIST, REG_FULL_RESOURCE_DESCRIPTOR, REG_QWORD. ")),
        };

        Ok(true)
    }
}
