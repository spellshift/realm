use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use spin::RwLock;

/// `enumerate(iterable, start=0)`: Returns an enumerate object.
///
/// Returns a list of tuples containing (index, value) pairs.
///
/// **Parameters**
/// - `iterable` (Iterable): The sequence to enumerate.
/// - `start` (Int): The starting index. Defaults to 0.
pub fn builtin_enumerate(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("enumerate() takes at least one argument".to_string());
    }
    let iterable = &args[0];
    let start = if args.len() > 1 {
        match args[1] {
            Value::Int(i) => i,
            _ => return Err("enumerate() start must be an integer".to_string()),
        }
    } else {
        0
    };
    let items = match iterable {
        Value::List(l) => l.read().clone(),
        Value::Tuple(t) => t.clone(),
        Value::Set(s) => s.read().iter().cloned().collect(),
        Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
        _ => {
            return Err(format!(
                "Type '{:?}' is not iterable",
                get_type_name(iterable)
            ));
        }
    };
    let mut pairs = Vec::new();
    for (i, item) in items.into_iter().enumerate() {
        pairs.push(Value::Tuple(vec![Value::Int(i as i64 + start), item]));
    }
    Ok(Value::List(Arc::new(RwLock::new(pairs))))
}
