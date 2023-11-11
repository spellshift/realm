use anyhow::Result;
use starlark::{values::{dict::Dict, Heap, Value}, collections::SmallMap};
use winreg::{{enums::*}, RegKey};

pub fn get_reg(starlark_heap: &Heap, reghive: String, regpath: String) -> Result<Dict>  {
    
    let res: SmallMap<Value, Value> = SmallMap::new();
    let mut tmp_res = Dict::new(res);
    

    #[cfg(not(target_os = "windows"))]
        return Err(anyhow::anyhow!("This OS isn't supported by the get_reg function.\nOnly windows systems are supported"));

    #[cfg(target_os = "windows")]
        //Accepted values for reghive :
        //HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, HKEY_USERS, HKEY_PERFORMANCE_DATA, HKEY_PERFORMANCE_TEXT, HKEY_PERFORMANCE_NLSTEXT, HKEY_CURRENT_CONFIG, HKEY_DYN_DATA, HKEY_CURRENT_USER_LOCAL_SETTINGS
        let mut ihive : isize = 0;

        if reghive == "HKEY_CLASSES_ROOT" {
            ihive = HKEY_CLASSES_ROOT;
        }
        else if reghive == "HKEY_CURRENT_USER" {
            ihive = HKEY_CURRENT_USER;
        }
        else if reghive == "HKEY_LOCAL_MACHINE" {
            ihive = HKEY_LOCAL_MACHINE;
        }
        else if reghive == "HKEY_USERS" {
            ihive = HKEY_USERS;
        }
        else if reghive == "HKEY_PERFORMANCE_DATA" {
            ihive = HKEY_PERFORMANCE_DATA;
        }
        else if reghive == "HKEY_PERFORMANCE_TEXT" {
            ihive = HKEY_PERFORMANCE_TEXT;
        }
        else if reghive == "HKEY_PERFORMANCE_NLSTEXT" {
            ihive = HKEY_PERFORMANCE_NLSTEXT;
        }
        else if reghive == "HKEY_CURRENT_CONFIG" {
            ihive = HKEY_CURRENT_CONFIG;
        }
        else if reghive == "HKEY_DYN_DATA" {
            ihive = HKEY_DYN_DATA;
        }
        else if reghive == "HKEY_CURRENT_USER_LOCAL_SETTINGS" {
            ihive = HKEY_CURRENT_USER_LOCAL_SETTINGS;
        }
        else{
            return Err(anyhow::anyhow!("RegHive can only be one of the following values - HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, HKEY_USERS, HKEY_PERFORMANCE_DATA, HKEY_PERFORMANCE_TEXT, HKEY_PERFORMANCE_NLSTEXT, HKEY_CURRENT_CONFIG, HKEY_DYN_DATA, HKEY_CURRENT_USER_LOCAL_SETTINGS "));
        }
        
        
        let hive = RegKey::predef(ihive);
        let subkey = hive.open_subkey(regpath)?;

        
        for (key, val) in subkey.enum_values().map(|x| x.unwrap()) {
            let key_value = starlark_heap.alloc_str(&key.to_string());
            let val_value = starlark_heap.alloc_str(&val.to_string());
        	tmp_res.insert_hashed(
                match key_value.to_value().get_hashed() {
                    Ok(val) => val,
                    Err(e) => return Err(anyhow::anyhow!("Failed to alloc name information: {}", e)),
                }
                val_value.to_value(),
            );
    	}

    Ok(tmp_res)
}

#[cfg(test)]
mod tests {
    use starlark::{values::{Value, Heap}, const_frozen_string};
    use super::*;


    #[test]
    fn test_get_reg() -> anyhow::Result<()> {
        #[cfg(not(target_os = "windows"))]
           return Err(anyhow::anyhow!("OS Not supported please re run on Windows"));

        #[cfg(target_os = "windows")]
            let binding = Heap::new();
            //Write something into temp regkey...
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let (nkey, _ndisp) = hkcu.create_subkey("SOFTWARE\\TEST1")?;
            nkey.set_value("FOO", &"BAR")?;

            let ares = get_reg(&binding, "HKEY_CURRENT_USER".to_string(), "SOFTWARE\\TEST1".to_string());
            let val2 : Value<'_> = ares?.get(const_frozen_string!("FOO").to_value())?.unwrap();
            //delete temp regkey
            nkey.delete_value("Foo")?;

            assert_eq!(val2.unpack_str().unwrap(), "BAR");

            

        Ok(())
    }
}

