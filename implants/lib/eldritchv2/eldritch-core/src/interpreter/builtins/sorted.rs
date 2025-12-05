use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::format;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::string::ToString;
use spin::RwLock;

pub fn builtin_sorted(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("sorted expected 1 argument, got 0".to_string());
    }
    if args.len() != 1 {
        // Ignoring key/reverse args for simplicity as per prompt description ("returns a new list... sorted order")
        return Err(format!("sorted expected 1 argument, got {}", args.len()));
    }

    let mut items = match &args[0] {
        Value::List(l) => l.read().clone(),
        Value::Tuple(t) => t.clone(),
        Value::Set(s) => s.read().iter().cloned().collect(),
        Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
        Value::Dictionary(d) => d
            .read()
            .keys()
            .cloned()
            .collect(),
        _ => {
            return Err(format!(
                "'{}' object is not iterable",
                get_type_name(&args[0])
            ))
        }
    };

    items.sort(); // uses Ord implementation on Value

    Ok(Value::List(Arc::new(RwLock::new(items))))
}
