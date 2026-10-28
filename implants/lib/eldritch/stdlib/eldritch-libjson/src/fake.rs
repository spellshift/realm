use super::JsonLibrary;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;
use spin::RwLock;

#[derive(Default, Debug)]
#[eldritch_library_impl(JsonLibrary)]
pub struct JsonLibraryFake;

impl JsonLibrary for JsonLibraryFake {
    fn decode(&self, content: String) -> Result<Value, String> {
        #[allow(clippy::mutable_key_type)]
        let mut map = BTreeMap::new();
        // Very basic fake parsing
        let trimmed = content.trim();
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            let inner = &trimmed[1..trimmed.len() - 1];
            for pair in inner.split(',') {
                if let Some((k, v)) = pair.split_once(':') {
                    let k = k.trim().trim_matches('"');
                    let v = v.trim().trim_matches('"');
                    map.insert(Value::String(k.to_string()), Value::String(v.to_string()));
                }
            }
        }
        if map.is_empty() {
            map.insert(
                Value::String("key".to_string()),
                Value::String("value".to_string()),
            );
            map.insert(Value::String("num".to_string()), Value::Int(123));
        }
        Ok(Value::Dictionary(Arc::new(RwLock::new(map))))
    }

    fn encode(&self, content: Value) -> Result<String, String> {
        Ok(format!("{}", content))
    }

    fn is_valid(&self, content: String) -> Result<bool, String> {
        let trimmed = content.trim();
        Ok((trimmed.starts_with('{') && trimmed.ends_with('}'))
            || (trimmed.starts_with('[') && trimmed.ends_with(']')))
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_fake_json() {
        let json = JsonLibraryFake;
        assert!(json.is_valid("{\"a\":1}".to_string()).unwrap());
        assert!(!json.is_valid("invalid".to_string()).unwrap());

        let json_str = "{\"hello\": \"world\"}".to_string();
        let val = json.decode(json_str).unwrap();
        if let Value::Dictionary(d) = val.clone() {
            let map = d.read();
            assert_eq!(
                map.get(&Value::String("hello".to_string())),
                Some(&Value::String("world".to_string()))
            );
        } else {
            panic!("Expected dictionary");
        }

        let json_out = json.encode(val).unwrap();
        assert!(json_out.contains("hello"));
    }
}
