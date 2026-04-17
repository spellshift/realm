use crate::ast::{Environment, Value};
use crate::interpreter::error::NativeError;
use crate::interpreter::introspection::get_type_name;
use alloc::format;
use alloc::sync::Arc;
use spin::RwLock;

/// `abs(x)`: Returns the absolute value of a number.
///
/// **Parameters**
/// - `x` (Int | Float): The number.
pub fn builtin_abs(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, NativeError> {
    if args.len() != 1 {
        return Err(NativeError::runtime_error(format!(
            "abs() takes exactly one argument ({} given)",
            args.len()
        )));
    }
    match &args[0] {
        Value::Int(i) => Ok(Value::Int(i.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err(NativeError::runtime_error(format!(
            "bad operand type for abs(): '{}'",
            get_type_name(&args[0])
        ))),
    }
}
