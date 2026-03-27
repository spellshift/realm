#[cfg(feature = "stdlib")]
use alloc::collections::BTreeMap;
#[cfg(feature = "stdlib")]
use alloc::string::String;
#[cfg(feature = "stdlib")]
use alloc::string::ToString;
#[cfg(feature = "stdlib")]
use anyhow::Result as AnyhowResult;
#[cfg(feature = "stdlib")]
use eldritch_core::Value;

#[cfg(feature = "stdlib")]
pub fn template_str(
    template: String,
    args: BTreeMap<String, Value>,
    autoescape: bool,
) -> AnyhowResult<String> {
    template_str_impl(template, args, autoescape)
}

#[cfg(feature = "stdlib")]
fn template_str_impl(
    template: String,
    args: BTreeMap<String, Value>,
    autoescape: bool,
) -> AnyhowResult<String> {
    use tera::{Context as TeraContext, Tera};

    let mut context = TeraContext::new();
    for (k, v) in args {
        // Convert Value to serde_json::Value
        let json_val = value_to_json(v);
        context.insert(k, &json_val);
    }

    let res_content = Tera::one_off(&template, &context, autoescape)?;
    Ok(res_content)
}

#[cfg(feature = "stdlib")]
fn value_to_json(v: Value) -> serde_json::Value {
    use alloc::format;
    use alloc::vec::Vec;
    use serde_json::Value as JsonValue;
    match v {
        Value::None => JsonValue::Null,
        Value::Bool(b) => JsonValue::Bool(b),
        Value::Int(i) => JsonValue::Number(serde_json::Number::from(i)),
        Value::Float(f) => serde_json::Number::from_f64(f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        Value::String(s) => JsonValue::String(s),
        Value::List(l) => {
            let list = l.read();
            let vec: Vec<JsonValue> = list.iter().map(|v| value_to_json(v.clone())).collect();
            JsonValue::Array(vec)
        }
        Value::Dictionary(d) => {
            let dict = d.read();
            let mut map = serde_json::Map::new();
            for (k, v) in dict.iter() {
                if let Value::String(key) = k {
                    map.insert(key.clone(), value_to_json(v.clone()));
                } else {
                    map.insert(k.to_string(), value_to_json(v.clone()));
                }
            }
            JsonValue::Object(map)
        }
        _ => JsonValue::String(format!("{v}")), // Fallback for types not easily mappable
    }
}

#[cfg(test)]
#[cfg(feature = "stdlib")]
mod tests {
    use super::*;

    #[test]
    fn test_template_str() {
        let mut args = BTreeMap::new();
        args.insert("name".to_string(), Value::String("World".to_string()));

        let result = template_str("Hello {{ name }}".to_string(), args, true).unwrap();

        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_template_multiple_args() {
        let mut args = BTreeMap::new();
        args.insert("name".to_string(), Value::String("Realm".to_string()));
        args.insert(
            "adjective".to_string(),
            Value::String("awesome".to_string()),
        );

        let result =
            template_str("{{ name }} is {{ adjective }}!".to_string(), args, true).unwrap();

        assert_eq!(result, "Realm is awesome!");
    }

    #[test]
    fn test_broken_template_str() {
        let mut args = BTreeMap::new();
        args.insert("name".to_string(), Value::String("World".to_string()));

        let result = template_str("Hello {% if {{ name }}".to_string(), args, true);

        assert!(result.is_err());
    }
}
