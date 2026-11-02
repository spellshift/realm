use crate::ast::{Environment, Value};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use spin::RwLock;

/// `str(object="")`: Returns a string containing a nicely printable representation of an object.
///
/// **Parameters**
/// - `object` (Any): The object to convert.
pub fn builtin_str(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::String(String::new()));
    }
    Ok(Value::String(args[0].to_string()))
}
