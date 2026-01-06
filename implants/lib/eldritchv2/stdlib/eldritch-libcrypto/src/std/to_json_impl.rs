use alloc::format;
use alloc::string::{String, ToString};
use eldritch_core::Value;

pub fn to_json(content: Value) -> Result<String, String> {
    let json_value = convert_value_to_json(&content)?;
    serde_json::to_string(&json_value)
        .map_err(|e| format!("Error serializing to json: {:?}", e))
}

fn convert_value_to_json(val: &Value) -> Result<serde_json::Value, String> {
    match val {
        Value::None => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Int(i) => Ok(serde_json::json!(i)),
        Value::Float(f) => Ok(serde_json::json!(f)),
        Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        Value::Bytes(_b) => {
            // Bytes are not natively JSON serializable.
            Err("Object of type 'bytes' is not JSON serializable".to_string())
        }
        Value::List(l) => {
            let list = l.read();
            let mut res = alloc::vec::Vec::with_capacity(list.len());
            for item in list.iter() {
                res.push(convert_value_to_json(item)?);
            }
            Ok(serde_json::Value::Array(res))
        }
        Value::Tuple(t) => {
            let mut res = alloc::vec::Vec::with_capacity(t.len());
            for item in t.iter() {
                res.push(convert_value_to_json(item)?);
            }
            Ok(serde_json::Value::Array(res))
        }
        Value::Dictionary(d) => {
            let dict = d.read();
            let mut res = serde_json::Map::new();
            for (k, v) in dict.iter() {
                if let Value::String(s) = k {
                    res.insert(s.clone(), convert_value_to_json(v)?);
                } else {
                    // JSON keys must be strings
                    return Err(format!("Keys must be strings, got {:?}", k));
                }
            }
            Ok(serde_json::Value::Object(res))
        }
        Value::Set(_) => Err("Object of type 'set' is not JSON serializable".to_string()),
        Value::Function(_)
        | Value::NativeFunction(_, _)
        | Value::NativeFunctionWithKwargs(_, _)
        | Value::BoundMethod(_, _)
        | Value::Foreign(_) => Err(format!(
            "Object of type '{:?}' is not JSON serializable",
            val
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeMap;
    use alloc::vec::Vec;
    use eldritch_core::conversion::ToValue;

    #[test]
    fn to_json_object() -> Result<(), String> {
        #[allow(clippy::mutable_key_type)]
        let mut map = BTreeMap::new();
        map.insert("test".to_string().to_value(), "test".to_string().to_value());
        let val = map.to_value();

        let res = to_json(val)?;
        assert_eq!(res, r#"{"test":"test"}"#);
        Ok(())
    }

    #[test]
    fn to_json_list() -> Result<(), String> {
        let vec_val: Vec<Value> = vec![
            1i64.to_value(),
            "foo".to_string().to_value(),
            false.to_value(),
            Value::None,
        ];
        let val = vec_val.to_value();

        let res = to_json(val)?;
        assert_eq!(res, r#"[1,"foo",false,null]"#);
        Ok(())
    }

    #[test]
    fn to_json_float() -> Result<(), String> {
        let val = Value::Float(13.37);
        let res = to_json(val)?;
        assert_eq!(res, "13.37");
        Ok(())
    }

    #[test]
    fn to_json_tuple() -> Result<(), String> {
        let t = vec![1i64.to_value(), 2i64.to_value()];
        let val = Value::Tuple(t);
        let res = to_json(val)?;
        assert_eq!(res, "[1,2]");
        Ok(())
    }

    #[test]
    fn to_json_invalid_bytes() {
        let val = Value::Bytes(vec![0xFF]);
        let res = to_json(val);
        assert!(res.is_err());
    }

    #[test]
    fn to_json_invalid_set() {
        let val = Value::Set(alloc::sync::Arc::new(spin::RwLock::new(
            alloc::collections::BTreeSet::new(),
        )));
        let res = to_json(val);
        assert!(res.is_err());
    }

    #[test]
    fn to_json_invalid_dict_keys() {
        #[allow(clippy::mutable_key_type)]
        let mut map = BTreeMap::new();
        map.insert(1i64.to_value(), "test".to_string().to_value());
        let val = map.to_value();

        let res = to_json(val);
        assert!(res.is_err());
    }

    #[test]
    fn to_json_invalid_function() {
        let val = Value::Set(alloc::sync::Arc::new(spin::RwLock::new(
            alloc::collections::BTreeSet::new(),
        )));
        assert!(to_json(val).is_err());
    }
}
