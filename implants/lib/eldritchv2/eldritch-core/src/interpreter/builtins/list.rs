use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::format;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use spin::RwLock;

/// `list([iterable])`: Creates a list.
///
/// If no argument is given, the constructor creates a new empty list.
/// The argument must be an iterable if specified.
pub fn builtin_list(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::List(Arc::new(RwLock::new(Vec::new()))));
    }
    if args.len() != 1 {
        return Err(format!(
            "list() takes at most 1 argument ({} given)",
            args.len()
        ));
    }

    let items = match &args[0] {
        Value::List(l) => l.read().clone(),
        Value::Tuple(t) => t.clone(),
        Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
        Value::Set(s) => s.read().iter().cloned().collect(),
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

    Ok(Value::List(Arc::new(RwLock::new(items))))
}
