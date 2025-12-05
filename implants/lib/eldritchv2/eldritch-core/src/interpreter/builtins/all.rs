use crate::ast::{Environment, Value};
use crate::interpreter::introspection::{get_type_name, is_truthy};
use alloc::format;
use alloc::sync::Arc;
use alloc::string::String;
use spin::RwLock;

pub fn builtin_all(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "all() takes exactly one argument ({} given)",
            args.len()
        ));
    }

    let items = match &args[0] {
        Value::List(l) => l.read().clone(),
        Value::Tuple(t) => t.clone(),
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

    for item in items {
        if !is_truthy(&item) {
            return Ok(Value::Bool(false));
        }
    }
    Ok(Value::Bool(true))
}
