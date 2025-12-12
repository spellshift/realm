use crate::ast::{Environment, Value};
use alloc::format;
use alloc::sync::Arc;
use alloc::string::String;
use spin::RwLock;

/// `len(s)`: Returns the length of an object.
///
/// The argument may be a sequence (such as a string, bytes, tuple, list, or range)
/// or a collection (such as a dictionary or set).
pub fn builtin_len(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "TypeError: len() takes exactly one argument ({} given)",
            args.len()
        ));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::Bytes(b) => Ok(Value::Int(b.len() as i64)),
        Value::List(l) => Ok(Value::Int(l.read().len() as i64)),
        Value::Dictionary(d) => Ok(Value::Int(d.read().len() as i64)),
        Value::Tuple(t) => Ok(Value::Int(t.len() as i64)),
        Value::Set(s) => Ok(Value::Int(s.read().len() as i64)),
        _ => Err(format!("TypeError: object of type '{}' has no len()", crate::interpreter::introspection::get_type_name(&args[0]))),
    }
}
