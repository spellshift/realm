use anyhow::Result;
use starlark::{ values::{Heap, Value} };


pub fn write_reg(starlark_heap: &Heap, reghive: String, regpath: String, regname: String, regvalue: String ) -> Result<String>  {
    
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
        
        let hive = RegKey::predef(ihive);
        let (nkey, ndisp) = hive.create_subkey(regpath)?;

        let resp : String = match ndisp {
            REG_CREATED_NEW_KEY => "A new key has been created",
            REG_OPENED_EXISTING_KEY => "An existing key has been modified",
        };

        nkey.set_value(regname, regvalue)?;
        match resp {
            Ok(_) => Ok(resp.to_string()),
            Err(_) => return Err(anyhow::anyhow!("There was an error executing the write_reg function.")),
        }
    }
}

#[cfg(test)]
mod tests {
    use starlark::{values::{Value, Heap}, const_frozen_string};
    use super::*;

    #[test]
    fn test_write_reg() -> anyhow::Result<()> {

        #[cfg(target_os = "windows")]{
            use winreg::{{enums::*}, RegKey};
            let binding = Heap::new();
            //Write something into temp regkey...
            let ares = write_reg(&binding, "HKEY_CURRENT_USER".to_string(), "SOFTWARE\\TEST1".to_string(),"FOO".to_string(),"BAR".to_string());
            
            //read temp regkey
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let subky = hkcu.open_subkey("SOFTWARE\\TEST1")?;
            let val2: String = subky.get_value("FOO")?;
            
            //delete temp regkey
            subky.delete_value("FOO")?;

            assert_eq!(val2.unpack_str().unwrap(), "BAR");
    
        }

        Ok(())
    }
}

