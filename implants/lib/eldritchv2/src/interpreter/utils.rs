use crate::ast::Value;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;

pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::None => false,
        Value::Bool(b) => *b,
        Value::Int(i) => *i != 0,
        Value::String(s) => !s.is_empty(),
        Value::Bytes(b) => !b.is_empty(),
        Value::List(l) => !l.borrow().is_empty(),
        Value::Dictionary(d) => !d.borrow().is_empty(),
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
        Value::String(_) => "string".to_string(),
        Value::Bytes(_) => "bytes".to_string(),
        Value::List(_) => "list".to_string(),
        Value::Dictionary(_) => "dict".to_string(),
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

pub fn adjust_slice_indices(
    length: i64,
    start: &Option<i64>,
    stop: &Option<i64>,
    step: i64,
) -> (i64, i64) {
    let start_val = if let Some(s) = start {
        let mut s = *s;
        if s < 0 {
            s += length;
        }
        if step < 0 {
            if s >= length {
                length - 1
            } else if s < 0 {
                -1
            } else {
                s
            }
        } else {
            if s < 0 {
                0
            } else if s > length {
                length
            } else {
                s
            }
        }
    } else {
        if step < 0 {
            length - 1
        } else {
            0
        }
    };

    let stop_val = if let Some(s) = stop {
        let mut s = *s;
        if s < 0 {
            s += length;
        }
        if step < 0 {
            if s < -1 {
                -1
            } else if s >= length {
                length - 1
            } else {
                s
            }
        } else {
            if s < 0 {
                0
            } else if s > length {
                length
            } else {
                s
            }
        }
    } else {
        if step < 0 {
            -1
        } else {
            length
        }
    };

    (start_val, stop_val)
}

pub fn compare_values(a: &Value, b: &Value) -> Result<Ordering, String> {
    match (a, b) {
        (Value::None, Value::None) => Ok(Ordering::Equal),
        (Value::Bool(l), Value::Bool(r)) => Ok(l.cmp(r)),
        (Value::Int(l), Value::Int(r)) => Ok(l.cmp(r)),
        (Value::String(l), Value::String(r)) => Ok(l.cmp(r)),
        (Value::Bytes(l), Value::Bytes(r)) => Ok(l.cmp(r)),
        (Value::List(l), Value::List(r)) => {
            let l_vec = l.borrow();
            let r_vec = r.borrow();
            for (v1, v2) in l_vec.iter().zip(r_vec.iter()) {
                let ord = compare_values(v1, v2)?;
                if ord != Ordering::Equal {
                    return Ok(ord);
                }
            }
            Ok(l_vec.len().cmp(&r_vec.len()))
        }
        (Value::Tuple(l), Value::Tuple(r)) => {
            for (v1, v2) in l.iter().zip(r.iter()) {
                let ord = compare_values(v1, v2)?;
                if ord != Ordering::Equal {
                    return Ok(ord);
                }
            }
            Ok(l.len().cmp(&r.len()))
        }
        _ => Err(format!(
            "Type mismatch or unsortable types: {} <-> {}",
            get_type_name(a),
            get_type_name(b)
        )),
    }
}
