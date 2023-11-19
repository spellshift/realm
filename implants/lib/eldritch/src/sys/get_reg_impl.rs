use anyhow::Result;
#[cfg(target_os = "windows")]
use starlark::{values::{dict::Dict, Value}, collections::SmallMap};
use starlark::values::{Heap,dict::Dict};


#[cfg(not(target_os = "windows"))]
pub fn get_reg(_starlark_heap: &Heap, _reghive: String, _regpath: String) -> Result<Dict>  {
    return Err(anyhow::anyhow!("This OS isn't supported by the get_reg function. Only windows systems are supported"));
}

#[cfg(target_os = "windows")]
pub fn get_reg(starlark_heap: &Heap, reghive: String, regpath: String) -> Result<Dict>  {
    let res: SmallMap<Value, Value> = SmallMap::new();
    let mut tmp_res = Dict::new(res);

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
        let key_value = starlark_heap.alloc_str(&key.to_string());
        let val_value = starlark_heap.alloc_str(&val.to_string());
        tmp_res.insert_hashed(
            match key_value.to_value().get_hashed() {
                Ok(val) => val,
                Err(e) => return Err(anyhow::anyhow!("Failed to alloc name information: {}", e)),
            },
            val_value.to_value(),
        );
    }
    Ok(tmp_res)
}

#[cfg(target_os = "windows")]
#[cfg(test)]
mod tests {
    use starlark::{values::{Value, Heap}, const_frozen_string};
    use super::*;


    #[test]
    fn test_get_reg() -> anyhow::Result<()> {
        use winreg::{enums::HKEY_CURRENT_USER, RegKey};
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
