use crate::ast::{Environment, Value};
use alloc::format;
use alloc::sync::Arc;
use alloc::string::{String, ToString};
use spin::RwLock;

/// `str(object)`: Returns a string containing a nicely printable representation of an object.
///
/// **Parameters**
/// - `object` (Any): The object to convert.
pub fn builtin_str(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::String(String::new()));
    }
    if let Value::Bytes(b) = &args[0] {
        match String::from_utf8(b.clone()) {
            Ok(s) => Ok(Value::String(s)),
            Err(_) => Ok(Value::String(format!("{b:?}"))), // Fallback
        }
    } else {
        Ok(Value::String(args[0].to_string()))
    }
}
