use crate::ast::{Environment, Value};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use spin::RwLock;

/// `Exception([message, ...])`: Constructs an exception value.
///
/// **Parameters**
/// - `message` (Any, optional): The error message or details.
pub fn builtin_exception(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    match args.len() {
        0 => Ok(Value::String(String::new())),
        1 => Ok(Value::String(args[0].to_string())),
        _ => Ok(Value::String(Value::Tuple(args.to_vec()).to_string())),
    }
}
