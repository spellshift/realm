use super::super::ast::Value;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::None => false,
        Value::Bool(b) => *b,
        Value::Int(i) => *i != 0,
        Value::Float(f) => *f != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::Bytes(b) => !b.is_empty(),
        Value::List(l) => !l.read().is_empty(),
        Value::Dictionary(d) => !d.read().is_empty(),
        Value::Set(s) => !s.read().is_empty(),
        Value::Tuple(t) => !t.is_empty(),
        Value::Function(_)
        | Value::NativeFunction(_, _)
        | Value::NativeFunctionWithKwargs(_, _)
        | Value::BoundMethod(_, _)
        | Value::Foreign(_) => true,
    }
}

pub fn get_type_name(value: &Value) -> String {
    match value {
        Value::None => "NoneType".to_string(),
        Value::Bool(_) => "bool".to_string(),
        Value::Int(_) => "int".to_string(),
        Value::Float(_) => "float".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Bytes(_) => "bytes".to_string(),
        Value::List(_) => "list".to_string(),
        Value::Dictionary(_) => "dict".to_string(),
        Value::Set(_) => "set".to_string(),
        Value::Tuple(_) => "tuple".to_string(),
        Value::Function(_)
        | Value::NativeFunction(_, _)
        | Value::NativeFunctionWithKwargs(_, _)
        | Value::BoundMethod(_, _) => "function".to_string(),
        Value::Foreign(f) => f.type_name().to_string(),
    }
}

pub fn get_dir_attributes(value: &Value) -> Vec<String> {
    let mut attrs = match value {
        Value::List(_) => vec![
            "append".to_string(),
            "extend".to_string(),
            "index".to_string(),
            "insert".to_string(),
            "pop".to_string(),
            "remove".to_string(),
            "sort".to_string(),
        ],
        Value::Dictionary(_) => vec![
            "get".to_string(),
            "items".to_string(),
            "keys".to_string(),
            "popitem".to_string(),
            "update".to_string(),
            "values".to_string(),
        ],
        Value::Set(_) => vec![
            "add".to_string(),
            "clear".to_string(),
            "contains".to_string(), // not standard python but useful
            "difference".to_string(),
            "discard".to_string(),
            "intersection".to_string(),
            "isdisjoint".to_string(),
            "issubset".to_string(),
            "issuperset".to_string(),
            "pop".to_string(),
            "remove".to_string(),
            "symmetric_difference".to_string(),
            "union".to_string(),
            "update".to_string(),
        ],
        Value::String(_) => vec![
            "endswith".to_string(),
            "find".to_string(),
            "format".to_string(),
            "join".to_string(),
            "lower".to_string(),
            "replace".to_string(),
            "split".to_string(),
            "startswith".to_string(),
            "strip".to_string(),
            "upper".to_string(),
        ],
        Value::Foreign(f) => f.method_names(),
        _ => Vec::new(),
    };
    attrs.sort();
    attrs
}
