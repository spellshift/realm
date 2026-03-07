use alloc::string::{String, ToString};
#[cfg(target_os = "windows")]
use alloc::vec::Vec;
use anyhow::Result;
use eldritch_core::Value;

#[cfg(target_os = "windows")]
use crate::std::reg_utils::parse_registry_path;

#[allow(unused_variables)]
pub fn write_reg(path: String, regname: String, regtype: String, regvalue: Value) -> Result<bool> {
    #[cfg(not(target_os = "windows"))]
    return Err(anyhow::anyhow!(
        "This OS isn't supported by the write_reg function. Only windows systems are supported"
    ));

    #[cfg(target_os = "windows")]
    {
        use winreg::{enums::*, RegKey, RegValue};

        let (ihive, subkey_str) = parse_registry_path(&path)?;

        let hive = RegKey::predef(ihive);
        let (nkey, _ndisp) = hive.create_subkey(subkey_str)?;

        match regtype.as_ref() {
            "REG_NONE" => {
                let s = get_string_val(regvalue, "REG_NONE")?;
                nkey.set_value(regname, &s)?;
            }
            "REG_SZ" => {
                let s = get_string_val(regvalue, "REG_SZ")?;
                nkey.set_value(regname, &s)?;
            }
            "REG_EXPAND_SZ" => {
                let s = get_string_val(regvalue, "REG_EXPAND_SZ")?;
                nkey.set_value(regname, &s)?;
            }
            "REG_BINARY" => {
                let bytes = get_binary_val(regvalue, "REG_BINARY")?;
                let data = RegValue {
                    vtype: REG_BINARY,
                    bytes,
                };
                nkey.set_raw_value(regname, &data)?;
            }
            "REG_DWORD" => {
                let num = get_u32_val(regvalue, "REG_DWORD")?;
                let data = RegValue {
                    vtype: REG_DWORD,
                    bytes: num.to_le_bytes().to_vec(),
                };
                nkey.set_raw_value(regname, &data)?;
            }
            "REG_DWORD_BIG_ENDIAN" => {
                let num = get_u32_val(regvalue, "REG_DWORD_BIG_ENDIAN")?;
                let data = RegValue {
                    vtype: REG_DWORD_BIG_ENDIAN,
                    bytes: num.to_be_bytes().to_vec(),
                };
                nkey.set_raw_value(regname, &data)?;
            }
            "REG_LINK" => {
                let s = get_string_val(regvalue, "REG_LINK")?;
                nkey.set_value(regname, &s)?;
            }
            "REG_MULTI_SZ" => {
                let s = get_string_val(regvalue, "REG_MULTI_SZ")?;
                let parsed_value: Vec<&str> = s.split(',').collect();
                nkey.set_value(regname, &parsed_value)?;
            }
            "REG_RESOURCE_LIST" => {
                let bytes = get_binary_val(regvalue, "REG_RESOURCE_LIST")?;
                let data = RegValue {
                    vtype: REG_RESOURCE_LIST,
                    bytes,
                };
                nkey.set_raw_value(regname, &data)?;
            }
            "REG_FULL_RESOURCE_DESCRIPTOR" => {
                let bytes = get_binary_val(regvalue, "REG_FULL_RESOURCE_DESCRIPTOR")?;
                let data = RegValue {
                    vtype: REG_FULL_RESOURCE_DESCRIPTOR,
                    bytes,
                };
                nkey.set_raw_value(regname, &data)?;
            }
            "REG_RESOURCE_REQUIREMENTS_LIST" => {
                let bytes = get_binary_val(regvalue, "REG_RESOURCE_REQUIREMENTS_LIST")?;
                let data = RegValue {
                    vtype: REG_RESOURCE_REQUIREMENTS_LIST,
                    bytes,
                };
                nkey.set_raw_value(regname, &data)?;
            }
            "REG_QWORD" => {
                let num = get_u64_val(regvalue, "REG_QWORD")?;
                let data = RegValue {
                    vtype: REG_QWORD,
                    bytes: num.to_le_bytes().to_vec(),
                };
                nkey.set_raw_value(regname, &data)?;
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "RegType can only be one of the following values - REG_NONE, REG_SZ, REG_EXPAND_SZ, REG_BINARY, REG_DWORD, REG_DWORD_BIG_ENDIAN, REG_LINK, REG_MULTI_SZ, REG_RESOURCE_LIST, REG_FULL_RESOURCE_DESCRIPTOR, REG_RESOURCE_REQUIREMENTS_LIST, REG_QWORD."
                ));
            }
        };

        Ok(true)
    }
}

#[cfg(target_os = "windows")]
fn get_string_val(val: Value, expected_type: &str) -> Result<String> {
    match val {
        Value::String(s) => Ok(s.to_string()),
        _ => Err(anyhow::anyhow!(
            "Expected a string value for {}",
            expected_type
        )),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(target_os = "windows")]
    fn test_write_reg() -> anyhow::Result<()> {
        use super::*;
        use alloc::format;
        use std::str;
        use uuid::Uuid;
        use winreg::{enums::*, RegKey};

        let id = Uuid::new_v4();

        // -------------------- WRITE_REG TESTS ---------------------------------------

        // Write and then read REG_SZ into temp regkey...
        let mut _ares = write_reg(
            format!("HKCU\\SOFTWARE\\{}", id),
            "FOO_STR".to_string(),
            "REG_SZ".to_string(),
            Value::String("BAR2".to_string()),
        );
        let mut hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let mut subky = hkcu.open_subkey(format!("SOFTWARE\\{}", id))?;
        let mut val2 = subky.get_raw_value("FOO_STR")?;
        assert_eq!(val2.to_string(), "BAR2");
        hkcu.delete_subkey(format!("SOFTWARE\\{}", id))?;

        // Write and then read REG_DWORD into temp regkey...
        _ares = write_reg(
            format!("HKCU\\SOFTWARE\\{}", id),
            "FOO_INT".to_string(),
            "REG_DWORD".to_string(),
            Value::Int(12345678),
        );
        hkcu = RegKey::predef(HKEY_CURRENT_USER);
        subky = hkcu.open_subkey(format!("SOFTWARE\\{}", id))?;
        val2 = subky.get_raw_value("FOO_INT")?;
        assert_eq!(val2.bytes, 12345678u32.to_le_bytes().to_vec());
        hkcu.delete_subkey(format!("SOFTWARE\\{}", id))?;

        // Write and then read REG_BINARY into temp regkey...
        _ares = write_reg(
            format!("HKCU\\SOFTWARE\\{}", id),
            "FOO_BIN".to_string(),
            "REG_BINARY".to_string(),
            Value::String("deadbeef".to_string()),
        );
        hkcu = RegKey::predef(HKEY_CURRENT_USER);
        subky = hkcu.open_subkey(format!("SOFTWARE\\{}", id))?;
        val2 = subky.get_raw_value("FOO_BIN")?;
        assert_eq!(hex::encode(val2.bytes), "deadbeef");
        hkcu.delete_subkey(format!("SOFTWARE\\{}", id))?;

        Ok(())
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_write_reg_non_windows() {
        use eldritch_core::Value;
        let res = super::write_reg(
            "HKCU\\SOFTWARE".into(),
            "foo".into(),
            "REG_SZ".into(),
            Value::String("bar".to_string()),
        );
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .to_string()
                .contains("Only windows systems are supported")
        );
    }
}

#[cfg(target_os = "windows")]
fn get_binary_val(val: Value, expected_type: &str) -> Result<Vec<u8>> {
    match val {
        Value::String(s) => hex::decode(s.as_str()).map_err(|_| {
            anyhow::anyhow!(
                "Expected a valid hex string for binary type {}",
                expected_type
            )
        }),
        Value::Bytes(b) => Ok(b.to_vec()),
        Value::Int(i) => Ok((i as u32).to_le_bytes().to_vec()),
        _ => Err(anyhow::anyhow!(
            "Expected a hex string or int/bytes for {}",
            expected_type
        )),
    }
}

#[cfg(target_os = "windows")]
fn get_u32_val(val: Value, expected_type: &str) -> Result<u32> {
    match val {
        Value::Int(i) => Ok(i as u32),
        Value::String(s) => {
            // try to parse as hex if possible?
            if let Ok(num) = u32::from_str_radix(&s, 16) {
                Ok(num)
            } else if let Ok(num) = s.parse::<u32>() {
                Ok(num)
            } else {
                Err(anyhow::anyhow!("Failed to parse string to u32 for {}", expected_type))
            }
        }
        _ => Err(anyhow::anyhow!(
            "Expected an integer or string value for {}",
            expected_type
        )),
    }
}

#[cfg(target_os = "windows")]
fn get_u64_val(val: Value, expected_type: &str) -> Result<u64> {
    match val {
        Value::Int(i) => Ok(i as u64),
        Value::String(s) => {
            // try to parse as hex if possible
            if let Ok(num) = u64::from_str_radix(&s, 16) {
                Ok(num)
            } else if let Ok(num) = s.parse::<u64>() {
                Ok(num)
            } else {
                Err(anyhow::anyhow!("Failed to parse string to u64 for {}", expected_type))
            }
        }
        _ => Err(anyhow::anyhow!(
            "Expected an integer or string value for {}",
            expected_type
        )),
    }
}
