use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

/// `max(iterable)` or `max(arg1, arg2, *args)`: Returns the largest item.
///
/// **Parameters**
/// - `iterable` (Iterable): An iterable to search.
/// - `arg1, arg2, *args` (Any): Two or more arguments to compare.
pub fn builtin_max(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("max expected at least 1 argument, got 0".to_string());
    }

    let items: Vec<Value> = if args.len() == 1 {
        match &args[0] {
            Value::List(l) => l.read().clone(),
            Value::Tuple(t) => t.clone(),
            Value::Set(s) => s.read().iter().cloned().collect(),
            Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
            Value::Dictionary(d) => d.read().keys().cloned().collect(),
            _ => {
                return Err(format!(
                    "'{}' object is not iterable",
                    get_type_name(&args[0])
                ));
            }
        }
    } else {
        args.to_vec()
    };

    if items.is_empty() {
        return Err("max() arg is an empty sequence".to_string());
    }

    let mut max_val = &items[0];
    for item in &items[1..] {
        if item > max_val {
            max_val = item;
        }
    }

    Ok(max_val.clone())
}
