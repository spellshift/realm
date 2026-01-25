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
pub fn template(
    template_path: String,
    dst: String,
    args: BTreeMap<String, Value>,
    autoescape: bool,
) -> Result<(), String> {
    template_impl(template_path, dst, args, autoescape).map_err(|e| e.to_string())
}

#[cfg(not(feature = "stdlib"))]
pub fn template(
    _template_path: alloc::string::String,
    _dst: alloc::string::String,
    _args: alloc::collections::BTreeMap<alloc::string::String, eldritch_core::Value>,
    _autoescape: bool,
) -> Result<(), alloc::string::String> {
    Err("template requires stdlib feature".into())
}

#[cfg(feature = "stdlib")]
fn template_impl(
    template_path: String,
    dst: String,
    args: BTreeMap<String, Value>,
    autoescape: bool,
) -> AnyhowResult<()> {
    use std::fs;
    use tera::{Context as TeraContext, Tera};

    let mut context = TeraContext::new();
    for (k, v) in args {
        // Convert Value to serde_json::Value
        let json_val = value_to_json(v);
        context.insert(k, &json_val);
    }

    let data = fs::read(&template_path)?;
    let template_content = String::from_utf8_lossy(&data);

    let res_content = Tera::one_off(&template_content, &context, autoescape)?;
    fs::write(&dst, res_content)?;
    Ok(())
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
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_template() {
        let tmp_tmpl = NamedTempFile::new().unwrap();
        let tmpl_path = tmp_tmpl.path().to_string_lossy().to_string();

        fs::write(&tmpl_path, "Hello {{ name }}").unwrap();

        let tmp_out = NamedTempFile::new().unwrap();
        let out_path = tmp_out.path().to_string_lossy().to_string();

        let mut args = BTreeMap::new();
        args.insert("name".to_string(), Value::String("World".to_string()));

        template(tmpl_path, out_path.clone(), args, true).unwrap();

        assert_eq!(fs::read_to_string(&out_path).unwrap(), "Hello World");
    }
}
