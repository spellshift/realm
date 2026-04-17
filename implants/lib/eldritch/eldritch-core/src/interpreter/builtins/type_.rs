use crate::ast::{Environment, Value};
use crate::interpreter::error::NativeError;
use crate::interpreter::introspection::get_type_name;
use alloc::sync::Arc;
use spin::RwLock;

/// `type(object)`: Returns the type of the object.
///
/// Returns a string representation of the type of the object.
pub fn builtin_type(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, NativeError> {
    if args.len() != 1 {
        return Err(NativeError::type_error("type() takes exactly 1 argument"));
    }
    Ok(Value::String(get_type_name(&args[0]).to_string()))
}
