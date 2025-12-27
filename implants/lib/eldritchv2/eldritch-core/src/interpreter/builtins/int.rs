use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use spin::RwLock;

/// `int(x)`: Converts a number or string to an integer.
///
/// If x is a number, return x.__int__(). For floating point numbers, this truncates towards zero.
/// If x is not a number or if base is given, then x must be a string, bytes, or bytearray instance representing an integer literal in the given base.
pub fn builtin_int(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::Int(0));
    }
    if args.len() != 1 {
        return Err(format!(
            "int() takes at most 1 argument ({} given)",
            args.len()
        ));
    }
    match &args[0] {
        Value::Int(i) => Ok(Value::Int(*i)),
        Value::Float(f) => Ok(Value::Int(*f as i64)), // Truncate
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        Value::String(s) => s
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| format!("invalid literal for int(): '{s}'")),
        _ => Err(format!(
            "int() argument must be a string, bytes or number, not '{}'",
            get_type_name(&args[0])
        )),
    }
}
