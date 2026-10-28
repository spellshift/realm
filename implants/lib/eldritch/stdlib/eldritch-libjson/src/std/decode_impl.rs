use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_core::conversion::ToValue;

pub fn from_json(content: String) -> Result<Value, String> {
    let json_data: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Error parsing json: {:?}", e))?;
    convert_json_to_value(json_data)
}

#[allow(clippy::mutable_key_type)]
fn convert_json_to_value(json: serde_json::Value) -> Result<Value, String> {
    match json {
        serde_json::Value::Null => Ok(Value::None),
        serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err(format!("Unsupported number type: {n}"))
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(s)),
        serde_json::Value::Array(arr) => {
            let mut res = Vec::with_capacity(arr.len());
            for item in arr {
                res.push(convert_json_to_value(item)?);
            }
            Ok(res.to_value())
        }
        serde_json::Value::Object(map) => {
            let mut res = BTreeMap::new();
            for (k, v) in map {
                res.insert(Value::String(k), convert_json_to_value(v)?);
            }
            Ok(res.to_value())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeMap;
    use eldritch_core::conversion::ToValue;

    #[test]
    fn test_from_json_object() -> Result<(), String> {
        let res = from_json(r#"{"test": "test"}"#.to_string())?;
        #[allow(clippy::mutable_key_type)]
        let mut map = BTreeMap::new();
        map.insert("test".to_string().to_value(), "test".to_string().to_value());
        let expected = map.to_value();

        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn test_from_json_list() -> Result<(), String> {
        let res = from_json(r#"[1, "foo", false, null]"#.to_string())?;

        let vec = vec![
            1i64.to_value(),
            "foo".to_string().to_value(),
            false.to_value(),
            Value::None,
        ];
        let expected = vec.to_value();

        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn test_from_json_float() -> Result<(), String> {
        let res = from_json(r#"13.37"#.to_string())?;
        assert_eq!(res, Value::Float(13.37));
        Ok(())
    }

    #[test]
    fn test_from_json_invalid() {
        let res = from_json(r#"{"test":"#.to_string());
        assert!(res.is_err());
    }
}
