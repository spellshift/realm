use anyhow::Result;
use starlark::values::Heap;


pub fn write_reg(starlark_heap: &Heap, reghive: String, regpath: String, regname: String, regvalue: String ) -> Result<String>  {
    
    #[cfg(not(target_os = "windows"))]
        return Err(anyhow::anyhow!("This OS isn't supported by the write_reg function. Only windows systems are supported"));

    #[cfg(target_os = "windows")]{
        use winreg::{{enums::*}, RegKey};

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
        let (nkey, ndisp) = hive.create_subkey(regpath)?;

        let resp : String = match ndisp {
            REG_CREATED_NEW_KEY => "A new key has been created".to_string(),
            REG_OPENED_EXISTING_KEY => "An existing key has been modified".to_string(),
        };

        nkey.set_value(regname, &regvalue)?;
        Ok(resp.to_string())
    }
}

#[cfg(test)]
mod tests {
    use starlark::values::Heap;
    use super::*;

    #[test]
    fn test_write_reg() -> anyhow::Result<()> {

        #[cfg(target_os = "windows")]{
            use winreg::{{enums::*}, RegKey};
            let binding = Heap::new();
            //Write something into temp regkey...
            let ares = write_reg(&binding, "HKEY_CURRENT_USER".to_string(), "SOFTWARE\\TEST2".to_string(),"FOO2".to_string(),"BAR2".to_string());
            
            //read temp regkey
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let subky = hkcu.open_subkey("SOFTWARE\\TEST2")?;
            let val2: String = subky.get_value("FOO2")?;

            //delete temp regkey
            hkcu.delete_subkey("SOFTWARE\\TEST2")?;

            assert_eq!(val2, "BAR2");
    
        }

        Ok(())
    }
}

