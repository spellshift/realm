use alloc::string::String;
use anyhow::Result;
use eldritch_core::Value;

#[allow(unused_variables)]
pub fn write_reg(path: String, regname: String, regtype: String, regvalue: Value) -> Result<bool> {
    #[cfg(not(target_os = "windows"))]
    return Err(anyhow::anyhow!(
        "This OS isn't supported by the registry.write function. Only windows systems are supported"
    ));

    #[cfg(target_os = "windows")]
    {
        use super::reg_utils::parse_registry_path;
        use alloc::string::ToString;
        use alloc::vec::Vec;
        use winreg::{RegKey, RegValue, enums::*};

        let (ihive, subkey_path) = parse_registry_path(&path)?;

        let hive = RegKey::predef(ihive);
        let (nkey, _ndisp) = hive.create_subkey(subkey_path)?;

        match regtype.as_ref() {
            "REG_NONE" => {
                let data = parse_bytes_val(&regvalue, "REG_NONE")?;
                let val = RegValue {
                    vtype: REG_NONE,
                    bytes: data,
                };
                nkey.set_raw_value(regname, &val)?;
            }
            "REG_SZ" => {
                let s = match regvalue {
                    Value::String(s) => s,
                    Value::Bytes(b) => String::from_utf8(b)
                        .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in bytes for REG_SZ: {}", e))?,
                    _ => regvalue.to_string(),
                };
                nkey.set_value(regname, &s)?;
            }
            "REG_EXPAND_SZ" => {
                let s = match regvalue {
                    Value::String(s) => s,
                    Value::Bytes(b) => String::from_utf8(b).map_err(|e| {
                        anyhow::anyhow!("Invalid UTF-8 in bytes for REG_EXPAND_SZ: {}", e)
                    })?,
                    _ => regvalue.to_string(),
                };
                let bytes: Vec<u8> = s
                    .encode_utf16()
                    .chain(core::iter::once(0))
                    .flat_map(|v| v.to_le_bytes())
                    .collect();
                let val = RegValue {
                    vtype: REG_EXPAND_SZ,
                    bytes,
                };
                nkey.set_raw_value(regname, &val)?;
            }
            "REG_BINARY" => {
                let data = parse_bytes_val(&regvalue, "REG_BINARY")?;
                let val = RegValue {
                    vtype: REG_BINARY,
                    bytes: data,
                };
                nkey.set_raw_value(regname, &val)?;
            }
            "REG_DWORD" => {
                let parsed_value = parse_u32_val(&regvalue, "REG_DWORD")?;
                nkey.set_value(regname, &parsed_value)?;
            }
            "REG_DWORD_BIG_ENDIAN" => {
                let parsed_value = parse_u32_val(&regvalue, "REG_DWORD_BIG_ENDIAN")?;
                let val = RegValue {
                    vtype: REG_DWORD_BIG_ENDIAN,
                    bytes: parsed_value.to_be_bytes().to_vec(),
                };
                nkey.set_raw_value(regname, &val)?;
            }
            "REG_LINK" => {
                let s = match regvalue {
                    Value::String(s) => s,
                    Value::Bytes(b) => String::from_utf8(b).map_err(|e| {
                        anyhow::anyhow!("Invalid UTF-8 in bytes for REG_LINK: {}", e)
                    })?,
                    _ => regvalue.to_string(),
                };
                let bytes: Vec<u8> = s
                    .encode_utf16()
                    .chain(core::iter::once(0))
                    .flat_map(|v| v.to_le_bytes())
                    .collect();
                let val = RegValue {
                    vtype: REG_LINK,
                    bytes,
                };
                nkey.set_raw_value(regname, &val)?;
            }
            "REG_MULTI_SZ" => {
                let vec_items: Vec<String> = match regvalue {
                    Value::List(l) => l
                        .read()
                        .iter()
                        .map(|v| match v {
                            Value::String(s) => Ok(s.clone()),
                            Value::Bytes(b) => String::from_utf8(b.clone()).map_err(|e| {
                                anyhow::anyhow!("Invalid UTF-8 in bytes for REG_MULTI_SZ: {}", e)
                            }),
                            _ => Ok(v.to_string()),
                        })
                        .collect::<Result<Vec<String>>>()?,
                    Value::String(s) => s.split(',').map(|x| x.to_string()).collect(),
                    Value::Bytes(b) => {
                        let s = String::from_utf8(b).map_err(|e| {
                            anyhow::anyhow!("Invalid UTF-8 in bytes for REG_MULTI_SZ: {}", e)
                        })?;
                        s.split(',').map(|x| x.to_string()).collect()
                    }
                    _ => return Err(anyhow::anyhow!("Invalid value for REG_MULTI_SZ")),
                };
                nkey.set_value(regname, &vec_items)?;
            }
            "REG_RESOURCE_LIST" => {
                let data = parse_bytes_val(&regvalue, "REG_RESOURCE_LIST")?;
                let val = RegValue {
                    vtype: REG_RESOURCE_LIST,
                    bytes: data,
                };
                nkey.set_raw_value(regname, &val)?;
            }
            "REG_FULL_RESOURCE_DESCRIPTOR" => {
                let data = parse_bytes_val(&regvalue, "REG_FULL_RESOURCE_DESCRIPTOR")?;
                let val = RegValue {
                    vtype: REG_FULL_RESOURCE_DESCRIPTOR,
                    bytes: data,
                };
                nkey.set_raw_value(regname, &val)?;
            }
            "REG_RESOURCE_REQUIREMENTS_LIST" => {
                let data = parse_bytes_val(&regvalue, "REG_RESOURCE_REQUIREMENTS_LIST")?;
                let val = RegValue {
                    vtype: REG_RESOURCE_REQUIREMENTS_LIST,
                    bytes: data,
                };
                nkey.set_raw_value(regname, &val)?;
            }
            "REG_QWORD" => {
                let parsed_value = parse_u64_val(&regvalue, "REG_QWORD")?;
                nkey.set_value(regname, &parsed_value)?;
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "RegType can only be one of the following values - REG_NONE, REG_SZ, REG_EXPAND_SZ, REG_BINARY, REG_DWORD, REG_DWORD_BIG_ENDIAN, REG_LINK, REG_MULTI_SZ, REG_RESOURCE_LIST, REG_FULL_RESOURCE_DESCRIPTOR, REG_RESOURCE_REQUIREMENTS_LIST, REG_QWORD "
                ));
            }
        };

        Ok(true)
    }
}

#[cfg(target_os = "windows")]
fn parse_bytes_val(val: &Value, type_name: &str) -> Result<alloc::vec::Vec<u8>> {
    match val {
        Value::Bytes(b) => Ok(b.clone()),
        Value::String(s) => hex::decode(s.as_str()).map_err(|_| {
            anyhow::anyhow!(
                "Expected a valid hex string for binary type {}, got '{}'",
                type_name,
                s
            )
        }),
        Value::Int(i) => Ok(i.to_le_bytes().to_vec()),
        _ => Err(anyhow::anyhow!(
            "Expected a hex string or int/bytes for {}, got {:?}",
            type_name,
            val
        )),
    }
}

#[cfg(target_os = "windows")]
fn parse_u32_val(val: &Value, type_name: &str) -> Result<u32> {
    match val {
        Value::Int(i) => Ok(*i as u32),
        Value::String(s) => {
            if let Ok(v) = s.parse::<u32>() {
                Ok(v)
            } else {
                u32::from_str_radix(s.trim_start_matches("0x"), 16).map_err(|_| {
                    anyhow::anyhow!("Failed to parse string '{}' as u32 for {}", s, type_name)
                })
            }
        }
        Value::Bytes(b) => {
            let s = core::str::from_utf8(b)
                .map_err(|_| anyhow::anyhow!("Invalid UTF-8 in bytes for {}", type_name))?;
            if let Ok(v) = s.parse::<u32>() {
                Ok(v)
            } else {
                u32::from_str_radix(s.trim_start_matches("0x"), 16).map_err(|_| {
                    anyhow::anyhow!("Failed to parse bytes '{}' as u32 for {}", s, type_name)
                })
            }
        }
        _ => Err(anyhow::anyhow!(
            "Expected int or string for {}, got {:?}",
            type_name,
            val
        )),
    }
}

#[cfg(target_os = "windows")]
fn parse_u64_val(val: &Value, type_name: &str) -> Result<u64> {
    match val {
        Value::Int(i) => Ok(*i as u64),
        Value::String(s) => {
            if let Ok(v) = s.parse::<u64>() {
                Ok(v)
            } else {
                u64::from_str_radix(s.trim_start_matches("0x"), 16).map_err(|_| {
                    anyhow::anyhow!("Failed to parse string '{}' as u64 for {}", s, type_name)
                })
            }
        }
        Value::Bytes(b) => {
            let s = core::str::from_utf8(b)
                .map_err(|_| anyhow::anyhow!("Invalid UTF-8 in bytes for {}", type_name))?;
            if let Ok(v) = s.parse::<u64>() {
                Ok(v)
            } else {
                u64::from_str_radix(s.trim_start_matches("0x"), 16).map_err(|_| {
                    anyhow::anyhow!("Failed to parse bytes '{}' as u64 for {}", s, type_name)
                })
            }
        }
        _ => Err(anyhow::anyhow!(
            "Expected int or string for {}, got {:?}",
            type_name,
            val
        )),
    }
}

#[cfg(test)]
#[cfg(target_os = "windows")]
mod tests {
    use super::*;
    use alloc::format;
    use alloc::vec;
    use alloc::vec::Vec;
    use spin::RwLock;
    use std::sync::Arc;
    use uuid::Uuid;
    use winreg::{RegKey, enums::*};

    #[test]
    fn test_write_reg() -> anyhow::Result<()> {
        let id = Uuid::new_v4();
        let base_path = format!("HKEY_CURRENT_USER\\SOFTWARE\\Temp\\{}", id);
        let subkey_path = format!("SOFTWARE\\Temp\\{}", id);

        // Write and verify REG_SZ
        write_reg(
            base_path.clone(),
            "FOO".to_string(),
            "REG_SZ".to_string(),
            Value::String("BAR".to_string()),
        )?;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _disp) = hkcu.create_subkey(&subkey_path)?;
        let val: String = key.get_value("FOO")?;
        assert_eq!(val, "BAR");

        // Write and verify REG_NONE
        write_reg(
            base_path.clone(),
            "FOO2".to_string(),
            "REG_NONE".to_string(),
            Value::String("deadbeef".to_string()),
        )?;
        let val2 = key.get_raw_value("FOO2")?;
        assert_eq!(hex::encode(val2.bytes), "deadbeef");

        // Write and verify REG_EXPAND_SZ
        write_reg(
            base_path.clone(),
            "FOO3".to_string(),
            "REG_EXPAND_SZ".to_string(),
            Value::String("BAR2".to_string()),
        )?;
        let val3: String = key.get_value("FOO3")?;
        assert_eq!(val3, "BAR2");

        // Write and verify REG_BINARY (from hex string)
        write_reg(
            base_path.clone(),
            "FOO4".to_string(),
            "REG_BINARY".to_string(),
            Value::String("deadbeef".to_string()),
        )?;
        let val4 = key.get_raw_value("FOO4")?;
        assert_eq!(val4.bytes, vec![0xde, 0xad, 0xbe, 0xef]);

        // Write and verify REG_BINARY (from Bytes)
        write_reg(
            base_path.clone(),
            "FOO4_BYTES".to_string(),
            "REG_BINARY".to_string(),
            Value::Bytes(vec![1, 2, 3, 4]),
        )?;
        let val4_b = key.get_raw_value("FOO4_BYTES")?;
        assert_eq!(val4_b.bytes, vec![1, 2, 3, 4]);

        // Write and verify REG_DWORD (from Int)
        write_reg(
            base_path.clone(),
            "FOO5".to_string(),
            "REG_DWORD".to_string(),
            Value::Int(12345678),
        )?;
        let val5: u32 = key.get_value("FOO5")?;
        assert_eq!(val5, 12345678);

        // Write and verify REG_DWORD (from String)
        write_reg(
            base_path.clone(),
            "FOO5_STR".to_string(),
            "REG_DWORD".to_string(),
            Value::String("12345678".to_string()),
        )?;
        let val5_s: u32 = key.get_value("FOO5_STR")?;
        assert_eq!(val5_s, 12345678);

        // Write and verify REG_DWORD_BIG_ENDIAN
        write_reg(
            base_path.clone(),
            "FOO6".to_string(),
            "REG_DWORD_BIG_ENDIAN".to_string(),
            Value::Int(0x12345678),
        )?;
        let val6 = key.get_raw_value("FOO6")?;
        assert_eq!(val6.bytes, 0x12345678u32.to_be_bytes().to_vec());

        // Write and verify REG_LINK
        write_reg(
            base_path.clone(),
            "FOO7".to_string(),
            "REG_LINK".to_string(),
            Value::String("BAR2".to_string()),
        )?;
        let val7 = key.get_raw_value("FOO7")?;
        let expected_link: Vec<u8> = "BAR2"
            .encode_utf16()
            .chain(core::iter::once(0))
            .flat_map(|v| v.to_le_bytes())
            .collect();
        assert_eq!(val7.bytes, expected_link);

        // Write and verify REG_MULTI_SZ (from List)
        let list_val = Value::List(Arc::new(RwLock::new(vec![
            Value::String("A".to_string()),
            Value::String("B".to_string()),
        ])));
        write_reg(
            base_path.clone(),
            "FOO8".to_string(),
            "REG_MULTI_SZ".to_string(),
            list_val,
        )?;
        let val8: Vec<String> = key.get_value("FOO8")?;
        assert_eq!(val8, vec!["A".to_string(), "B".to_string()]);

        // Write and verify REG_MULTI_SZ (from String)
        write_reg(
            base_path.clone(),
            "FOO8_STR".to_string(),
            "REG_MULTI_SZ".to_string(),
            Value::String("C,D".to_string()),
        )?;
        let val8_s: Vec<String> = key.get_value("FOO8_STR")?;
        assert_eq!(val8_s, vec!["C".to_string(), "D".to_string()]);

        // Write and verify REG_RESOURCE_LIST
        write_reg(
            base_path.clone(),
            "FOO9".to_string(),
            "REG_RESOURCE_LIST".to_string(),
            Value::String("deadbeef".to_string()),
        )?;
        let val9 = key.get_raw_value("FOO9")?;
        assert_eq!(val9.bytes, vec![0xde, 0xad, 0xbe, 0xef]);

        // Write and verify REG_FULL_RESOURCE_DESCRIPTOR
        write_reg(
            base_path.clone(),
            "FOO10".to_string(),
            "REG_FULL_RESOURCE_DESCRIPTOR".to_string(),
            Value::String("deadbeef".to_string()),
        )?;
        let val10 = key.get_raw_value("FOO10")?;
        assert_eq!(val10.bytes, vec![0xde, 0xad, 0xbe, 0xef]);

        // Write and verify REG_RESOURCE_REQUIREMENTS_LIST
        write_reg(
            base_path.clone(),
            "FOO11".to_string(),
            "REG_RESOURCE_REQUIREMENTS_LIST".to_string(),
            Value::String("deadbeef".to_string()),
        )?;
        let val11 = key.get_raw_value("FOO11")?;
        assert_eq!(val11.bytes, vec![0xde, 0xad, 0xbe, 0xef]);

        // Write and verify REG_QWORD
        write_reg(
            base_path.clone(),
            "FOO12".to_string(),
            "REG_QWORD".to_string(),
            Value::Int(1234567812345678),
        )?;
        let val12: u64 = key.get_value("FOO12")?;
        assert_eq!(val12, 1234567812345678);

        // Delete temp regkey
        hkcu.delete_subkey(subkey_path)?;

        Ok(())
    }
}
