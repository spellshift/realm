use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use eldritch_core::Value;

/// Maximum nesting depth allowed when serializing Eldritch values to JSON.
const MAX_JSON_DEPTH: usize = 128;

pub fn to_json(content: Value) -> Result<String, String> {
    let mut visited = BTreeSet::new();
    let json_value = convert_value_to_json(&content, &mut visited, 0)?;
    serde_json::to_string(&json_value).map_err(|e| format!("Error serializing to json: {:?}", e))
}

fn convert_value_to_json(
    val: &Value,
    visited: &mut BTreeSet<usize>,
    depth: usize,
) -> Result<serde_json::Value, String> {
    if depth >= MAX_JSON_DEPTH {
        return Err("Maximum JSON serialization depth exceeded".to_string());
    }

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
            let ptr = Arc::as_ptr(l) as usize;
            if !visited.insert(ptr) {
                return Err("Circular reference detected during JSON serialization".to_string());
            }
            let list = l.read();
            let mut res = alloc::vec::Vec::with_capacity(list.len());
            let conv_res = (|| {
                for item in list.iter() {
                    res.push(convert_value_to_json(item, visited, depth + 1)?);
                }
                Ok(serde_json::Value::Array(res))
            })();
            visited.remove(&ptr);
            conv_res
        }
        Value::Tuple(t) => {
            let mut res = alloc::vec::Vec::with_capacity(t.len());
            for item in t.iter() {
                res.push(convert_value_to_json(item, visited, depth + 1)?);
            }
            Ok(serde_json::Value::Array(res))
        }
        Value::Dictionary(d) => {
            let ptr = Arc::as_ptr(d) as usize;
            if !visited.insert(ptr) {
                return Err("Circular reference detected during JSON serialization".to_string());
            }
            let dict = d.read();
            let mut res = serde_json::Map::new();
            let conv_res = (|| {
                for (k, v) in dict.iter() {
                    if let Value::String(s) = k {
                        res.insert(s.clone(), convert_value_to_json(v, visited, depth + 1)?);
                    } else {
                        // JSON keys must be strings
                        return Err(format!("Keys must be strings, got {:?}", k));
                    }
                }
                Ok(serde_json::Value::Object(res))
            })();
            visited.remove(&ptr);
            conv_res
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

    #[test]
    fn to_json_circular_list_returns_error() {
        let list = Arc::new(spin::RwLock::new(Vec::new()));
        list.write().push(Value::List(list.clone()));
        let err = to_json(Value::List(list)).unwrap_err();
        assert!(
            err.contains("Circular reference"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn to_json_circular_dict_returns_error() {
        #[allow(clippy::mutable_key_type)]
        let dict = Arc::new(spin::RwLock::new(BTreeMap::new()));
        dict.write().insert(
            Value::String("self".to_string()),
            Value::Dictionary(dict.clone()),
        );
        let err = to_json(Value::Dictionary(dict)).unwrap_err();
        assert!(
            err.contains("Circular reference"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn to_json_shared_non_cyclic_list_succeeds() -> Result<(), String> {
        let shared = Value::List(Arc::new(spin::RwLock::new(vec![1i64.to_value()])));
        let outer = Value::List(Arc::new(spin::RwLock::new(vec![shared.clone(), shared])));
        let res = to_json(outer)?;
        assert_eq!(res, "[[1],[1]]");
        Ok(())
    }

    #[test]
    fn to_json_excessive_nesting_returns_error() {
        let mut val = Value::Int(1);
        for _ in 0..(MAX_JSON_DEPTH + 5) {
            val = Value::Tuple(vec![val]);
        }
        let err = to_json(val).unwrap_err();
        assert!(err.contains("depth"), "unexpected error: {err}");
    }
}
