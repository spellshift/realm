use crate::ast::{Environment, Value};
use crate::interpreter::utils::get_type_name;
use alloc::format;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::string::ToString;
use spin::RwLock;

pub fn builtin_min(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("min expected 1 argument, got 0".to_string());
    }
    if args.len() != 1 {
        return Err(format!("min expected 1 argument, got {}", args.len()));
    }

    let items = match &args[0] {
        Value::List(l) => l.read().clone(),
        Value::Tuple(t) => t.clone(),
        Value::Set(s) => s.read().iter().cloned().collect(),
        Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
        Value::Dictionary(d) => d
            .read()
            .keys()
            .map(|k| Value::String(k.clone()))
            .collect(),
        _ => {
            return Err(format!(
                "'{}' object is not iterable",
                get_type_name(&args[0])
            ))
        }
    };

    if items.is_empty() {
        return Err("min() arg is an empty sequence".to_string());
    }

    let mut min_val = &items[0];
    for item in &items[1..] {
        if item < min_val {
            min_val = item;
        }
    }

    Ok(min_val.clone())
}
