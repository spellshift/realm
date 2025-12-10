use crate::ast::{Environment, Value};
use crate::interpreter::introspection::is_truthy;
use alloc::sync::Arc;
use alloc::string::String;
use spin::RwLock;

/// `bool(x)`: Converts a value to a Boolean.
///
/// Returns True when the argument x is true, False otherwise.
pub fn builtin_bool(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    Ok(Value::Bool(is_truthy(&args[0])))
}
