use anyhow::Result;


pub fn write_reg(reghive: String, regpath: String, regname: String, regtype:String, regvalue: String ) -> Result<String>  {
    
    #[cfg(not(target_os = "windows"))]
        return Err(anyhow::anyhow!("This OS isn't supported by the write_reg function. Only windows systems are supported"));

    #[cfg(target_os = "windows")]{
        use winreg::{{enums::*}, RegKey, RegValue};

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

        let rtype: RegType = match regtype.as_ref() {
            "REG_NONE" => REG_NONE,
            "REG_SZ" => REG_SZ,
            "REG_EXPAND_SZ" => REG_EXPAND_SZ,
            "REG_BINARY" => REG_BINARY,
            "REG_DWORD" => REG_DWORD,
            "REG_DWORD_BIG_ENDIAN" => REG_DWORD_BIG_ENDIAN,
            "REG_LINK" => REG_LINK,
            "REG_MULTI_SZ" => REG_MULTI_SZ,
            "REG_RESOURCE_LIST" => REG_RESOURCE_LIST,
            "REG_FULL_RESOURCE_DESCRIPTOR" => REG_FULL_RESOURCE_DESCRIPTOR,
            "REG_RESOURCE_REQUIREMENTS_LIST" => REG_RESOURCE_REQUIREMENTS_LIST,
            "REG_QWORD" => REG_QWORD,
            _ => return Err(anyhow::anyhow!("RegType can only be one of the following values - REG_NONE, REG_SZ, REG_EXPAND_SZ, REG_BINARY, REG_DWORD, REG_DWORD_BIG_ENDIAN, REG_LINK, REG_MULTI_SZ, REG_RESOURCE_LIST, REG_RESOURCE_LIST, REG_FULL_RESOURCE_DESCRIPTOR, REG_QWORD. ")),
        };
        
        let hive = RegKey::predef(ihive);
        let (nkey, ndisp) = hive.create_subkey(regpath)?;

        let resp : String = match ndisp {
            REG_CREATED_NEW_KEY => "A new key has been created".to_string(),
            REG_OPENED_EXISTING_KEY => "An existing key has been modified".to_string(),
        };

        let bytes = regvalue.as_bytes().to_vec();
        let data = RegValue{ vtype: rtype, bytes: bytes};
        nkey.set_raw_value(regname, &data)?;
        Ok(resp.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_write_reg() -> anyhow::Result<()> {

        #[cfg(target_os = "windows")]{
            use winreg::{{enums::*}, RegKey};
            let id = Uuid::new_v4();
            //Write something into temp regkey...
            let _ares = write_reg("HKEY_CURRENT_USER".to_string(), format!("SOFTWARE\\{}",id.to_string()).to_string(),"FOO2".to_string(), "REG_SZ".to_string(), "BAR2".to_string());
            
            //read temp regkey
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let subky = hkcu.open_subkey(format!("SOFTWARE\\{}",id.to_string()).to_string())?;
            let val2 = subky.get_raw_value("FOO2")?;

            //delete temp regkey
            //hkcu.delete_subkey(format!("SOFTWARE\\{}",id.to_string()).to_string())?;

            assert_eq!(val2.to_string(), "BAR2");
    
        }

        Ok(())
    }
}

