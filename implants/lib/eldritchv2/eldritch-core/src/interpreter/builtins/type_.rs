use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::string::String;
use alloc::sync::Arc;
use spin::RwLock;

/// `type(object)`: Returns the type of the object.
///
/// Returns a string representation of the type of the object.
pub fn builtin_type(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(String::from("TypeError: type() takes exactly 1 argument"));
    }
    Ok(Value::String(get_type_name(&args[0])))
}
