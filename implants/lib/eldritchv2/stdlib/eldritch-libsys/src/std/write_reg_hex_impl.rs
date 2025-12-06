#[cfg(target_os = "windows")]
use alloc::format;
use alloc::string::String;
#[cfg(target_os = "windows")]
use alloc::string::ToString;
#[cfg(target_os = "windows")]
use alloc::vec::Vec;
use anyhow::Result;

#[allow(unused_variables)]
pub fn write_reg_hex(
    reghive: String,
    regpath: String,
    regname: String,
    regtype: String,
    regvalue: String,
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
                let parsed_value: Vec<u8> = hex::decode(regvalue)?;
                let data = RegValue{ vtype: REG_BINARY, bytes: parsed_value};
                nkey.set_raw_value(regname, &data)?;
            },
            "REG_DWORD" => {
                let parsed_value: Vec<u8> = hex::decode(regvalue)?;
                let data = RegValue{ vtype: REG_DWORD, bytes: parsed_value};
                nkey.set_raw_value(regname, &data)?;
            },
            "REG_DWORD_BIG_ENDIAN" => {
                let parsed_value: u32 = u32::from_str_radix(&regvalue, 16)?;
                let data = RegValue{ vtype: REG_DWORD_BIG_ENDIAN, bytes: parsed_value.to_be_bytes().to_vec()};
                nkey.set_raw_value(regname, &data)?;
            },
            "REG_LINK" => {
                nkey.set_value(regname, &regvalue)?;
            },
            "REG_MULTI_SZ" => {
                let parsed_value: Vec<&str> = regvalue.split(',').collect();
                nkey.set_value(regname, &parsed_value)?;
            },
            "REG_RESOURCE_LIST" => {
                let parsed_value: Vec<u8> = hex::decode(regvalue)?;
                let data = RegValue{ vtype: REG_RESOURCE_LIST, bytes: parsed_value};
                nkey.set_raw_value(regname, &data)?;
            },
            "REG_FULL_RESOURCE_DESCRIPTOR" => {
                let parsed_value: Vec<u8> = hex::decode(regvalue)?;
                let data = RegValue{ vtype: REG_FULL_RESOURCE_DESCRIPTOR, bytes: parsed_value};
                nkey.set_raw_value(regname, &data)?;
            },
            "REG_RESOURCE_REQUIREMENTS_LIST" => {
                let parsed_value: Vec<u8> = hex::decode(regvalue)?;
                let data = RegValue{ vtype: REG_RESOURCE_REQUIREMENTS_LIST, bytes: parsed_value};
                nkey.set_raw_value(regname, &data)?;
            },
            "REG_QWORD" => {
                let parsed_value: u64 = u64::from_str_radix(&regvalue, 16)?;
                let data = RegValue{ vtype: REG_QWORD, bytes: parsed_value.to_le_bytes().to_vec()};
                nkey.set_raw_value(regname, &data)?;
            },
            _ => return Err(anyhow::anyhow!("RegType can only be one of the following values - REG_NONE, REG_SZ, REG_EXPAND_SZ, REG_BINARY, REG_DWORD, REG_DWORD_BIG_ENDIAN, REG_LINK, REG_MULTI_SZ, REG_RESOURCE_LIST, REG_RESOURCE_LIST, REG_FULL_RESOURCE_DESCRIPTOR, REG_QWORD. ")),
        };

        Ok(true)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_write_reg_hex() -> anyhow::Result<()> {
        #[cfg(target_os = "windows")]
        {
            use super::*;
            use uuid::Uuid;
            use winreg::{enums::*, RegKey};

            let id = Uuid::new_v4();

            // -------------------- WRITE_REG_HEX TESTS ---------------------------------------
            //Write and then read REG_SZ into temp regkey...
            let mut _ares = write_reg_hex(
                "HKEY_CURRENT_USER".to_string(),
                format!("SOFTWARE\\{}", id),
                "FOO2".to_string(),
                "REG_SZ".to_string(),
                "deadbeef".to_string(),
            );
            let mut hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let mut subky = hkcu.open_subkey(format!("SOFTWARE\\{}", id))?;
            let mut val2 = subky.get_raw_value("FOO2")?;
            assert_eq!(val2.to_string(), "deadbeef");

            //delete temp regkey
            hkcu.delete_subkey(format!("SOFTWARE\\{}", id))?;

            //Write and then read REG_NONE into temp regkey...
            _ares = write_reg_hex(
                "HKEY_CURRENT_USER".to_string(),
                format!("SOFTWARE\\{}", id),
                "FOO2".to_string(),
                "REG_NONE".to_string(),
                "deadbeef".to_string(),
            );
            hkcu = RegKey::predef(HKEY_CURRENT_USER);
            subky = hkcu.open_subkey(format!("SOFTWARE\\{}", id))?;
            val2 = subky.get_raw_value("FOO2")?;
            assert_eq!(val2.to_string(), "deadbeef");

            //delete temp regkey
            hkcu.delete_subkey(format!("SOFTWARE\\{}", id))?;

            //Write and then read REG_EXPAND_SZ into temp regkey...
            _ares = write_reg_hex(
                "HKEY_CURRENT_USER".to_string(),
                format!("SOFTWARE\\{}", id),
                "FOO2".to_string(),
                "REG_EXPAND_SZ".to_string(),
                "deadbeef".to_string(),
            );
            hkcu = RegKey::predef(HKEY_CURRENT_USER);
            subky = hkcu.open_subkey(format!("SOFTWARE\\{}", id))?;
            val2 = subky.get_raw_value("FOO2")?;
            assert_eq!(val2.to_string(), "deadbeef");

            //delete temp regkey
            hkcu.delete_subkey(format!("SOFTWARE\\{}", id))?;

            //Write and then read REG_BINARY into temp regkey...
            _ares = write_reg_hex(
                "HKEY_CURRENT_USER".to_string(),
                format!("SOFTWARE\\{}", id),
                "FOO2".to_string(),
                "REG_BINARY".to_string(),
                "deadbeef".to_string(),
            );
            hkcu = RegKey::predef(HKEY_CURRENT_USER);
            subky = hkcu.open_subkey(format!("SOFTWARE\\{}", id))?;
            val2 = subky.get_raw_value("FOO2")?;
            assert_eq!(hex::encode(val2.bytes), "deadbeef");

            //delete temp regkey
            hkcu.delete_subkey(format!("SOFTWARE\\{}", id))?;

            //Write and then read REG_DWORD into temp regkey...
            _ares = write_reg_hex(
                "HKEY_CURRENT_USER".to_string(),
                format!("SOFTWARE\\{}", id),
                "FOO2".to_string(),
                "REG_DWORD".to_string(),
                "deadbeef".to_string(),
            );
            hkcu = RegKey::predef(HKEY_CURRENT_USER);
            subky = hkcu.open_subkey(format!("SOFTWARE\\{}", id))?;
            val2 = subky.get_raw_value("FOO2")?;
            assert_eq!(hex::encode(val2.bytes), "deadbeef");

            //delete temp regkey
            hkcu.delete_subkey(format!("SOFTWARE\\{}", id))?;

            //Write and then read REG_DWORD_BIG_ENDIAN into temp regkey...
            _ares = write_reg_hex(
                "HKEY_CURRENT_USER".to_string(),
                format!("SOFTWARE\\{}", id),
                "FOO2".to_string(),
                "REG_DWORD_BIG_ENDIAN".to_string(),
                "deadbeef".to_string(),
            );
            hkcu = RegKey::predef(HKEY_CURRENT_USER);
            subky = hkcu.open_subkey(format!("SOFTWARE\\{}", id))?;
            val2 = subky.get_raw_value("FOO2")?;
            assert_eq!(val2.bytes, 0xdeadbeefu32.to_be_bytes().to_vec());

            //delete temp regkey
            hkcu.delete_subkey(format!("SOFTWARE\\{}", id))?;

            //Write and then read REG_LINK into temp regkey...
            _ares = write_reg_hex(
                "HKEY_CURRENT_USER".to_string(),
                format!("SOFTWARE\\{}", id),
                "FOO2".to_string(),
                "REG_LINK".to_string(),
                "deadbeef".to_string(),
            );
            hkcu = RegKey::predef(HKEY_CURRENT_USER);
            subky = hkcu.open_subkey(format!("SOFTWARE\\{}", id))?;
            val2 = subky.get_raw_value("FOO2")?;
            assert_eq!(val2.to_string(), "deadbeef");

            //delete temp regkey
            hkcu.delete_subkey(format!("SOFTWARE\\{}", id))?;

            //Write and then read REG_MULTI_SZ into temp regkey...
            _ares = write_reg_hex(
                "HKEY_CURRENT_USER".to_string(),
                format!("SOFTWARE\\{}", id),
                "FOO2".to_string(),
                "REG_MULTI_SZ".to_string(),
                "deadbeef".to_string(),
            );
            hkcu = RegKey::predef(HKEY_CURRENT_USER);
            subky = hkcu.open_subkey(format!("SOFTWARE\\{}", id))?;
            val2 = subky.get_raw_value("FOO2")?;
            assert_eq!(val2.to_string(), "deadbeef");

            //delete temp regkey
            hkcu.delete_subkey(format!("SOFTWARE\\{}", id))?;

            //Write and then read REG_RESOURCE_LIST into temp regkey...
            _ares = write_reg_hex(
                "HKEY_CURRENT_USER".to_string(),
                format!("SOFTWARE\\{}", id),
                "FOO2".to_string(),
                "REG_RESOURCE_LIST".to_string(),
                "deadbeef".to_string(),
            );
            hkcu = RegKey::predef(HKEY_CURRENT_USER);
            subky = hkcu.open_subkey(format!("SOFTWARE\\{}", id))?;
            val2 = subky.get_raw_value("FOO2")?;
            assert_eq!(hex::encode(val2.bytes), "deadbeef");

            //delete temp regkey
            hkcu.delete_subkey(format!("SOFTWARE\\{}", id))?;

            //Write and then read REG_FULL_RESOURCE_DESCRIPTOR into temp regkey...
            _ares = write_reg_hex(
                "HKEY_CURRENT_USER".to_string(),
                format!("SOFTWARE\\{}", id),
                "FOO2".to_string(),
                "REG_FULL_RESOURCE_DESCRIPTOR".to_string(),
                "deadbeef".to_string(),
            );
            hkcu = RegKey::predef(HKEY_CURRENT_USER);
            subky = hkcu.open_subkey(format!("SOFTWARE\\{}", id))?;
            val2 = subky.get_raw_value("FOO2")?;
            assert_eq!(hex::encode(val2.bytes), "deadbeef");

            //delete temp regkey
            hkcu.delete_subkey(format!("SOFTWARE\\{}", id))?;

            //Write and then read REG_RESOURCE_REQUIREMENTS_LIST into temp regkey...
            _ares = write_reg_hex(
                "HKEY_CURRENT_USER".to_string(),
                format!("SOFTWARE\\{}", id),
                "FOO2".to_string(),
                "REG_RESOURCE_REQUIREMENTS_LIST".to_string(),
                "deadbeef".to_string(),
            );
            hkcu = RegKey::predef(HKEY_CURRENT_USER);
            subky = hkcu.open_subkey(format!("SOFTWARE\\{}", id))?;
            val2 = subky.get_raw_value("FOO2")?;
            assert_eq!(hex::encode(val2.bytes), "deadbeef");

            //delete temp regkey
            hkcu.delete_subkey(format!("SOFTWARE\\{}", id))?;

            //Write and then read REG_QWORD into temp regkey...
            _ares = write_reg_hex(
                "HKEY_CURRENT_USER".to_string(),
                format!("SOFTWARE\\{}", id),
                "FOO2".to_string(),
                "REG_QWORD".to_string(),
                "deadbeefdeadbeef".to_string(),
            );
            hkcu = RegKey::predef(HKEY_CURRENT_USER);
            subky = hkcu.open_subkey(format!("SOFTWARE\\{}", id))?;
            val2 = subky.get_raw_value("FOO2")?;
            assert_eq!(val2.bytes, 0xdeadbeefdeadbeefu64.to_le_bytes().to_vec());

            //delete temp regkey
            hkcu.delete_subkey(format!("SOFTWARE\\{}", id))?;
        }

        Ok(())
    }
}
