use crate::ast::{Environment, Value};
use crate::interpreter::error::NativeError;
use crate::interpreter::introspection::is_truthy;
use alloc::format;
use alloc::sync::Arc;
use spin::RwLock;

/// `assert(condition)`: Aborts if the condition is false.
///
/// **Parameters**
/// - `condition` (Any): The condition to check.
pub fn builtin_assert(
    _env: &Arc<RwLock<Environment>>,
    args: &[Value],
) -> Result<Value, NativeError> {
    if args.len() != 1 {
        return Err(NativeError::runtime_error(format!(
            "assert() takes exactly one argument ({} given)",
            args.len()
        )));
    }
    if !is_truthy(&args[0]) {
        return Err(NativeError::runtime_error(format!(
            "Assertion failed: value '{:?}' is not truthy",
            args[0]
        )));
    }
    Ok(Value::None)
}
