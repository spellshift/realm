use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::format;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::string::ToString;
use spin::RwLock;

pub fn builtin_reversed(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "reversed() takes exactly one argument ({} given)",
            args.len()
        ));
    }

    let items = match &args[0] {
        Value::List(l) => l.read().clone(),
        Value::Tuple(t) => t.clone(),
        Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
        // Dictionary and Set are not reversible in Python (TypeError)
        _ => {
            return Err(format!(
                "'{}' object is not reversible",
                get_type_name(&args[0])
            ))
        }
    };

    let mut rev_items = items;
    rev_items.reverse();

    // Python reversed() returns an iterator. Here we return a List (per prompt: "returns a new list").
    Ok(Value::List(Arc::new(RwLock::new(rev_items))))
}
