use crate::ast::{Environment, Value};
use crate::interpreter::error::NativeError;
use alloc::format;
use alloc::sync::Arc;
use spin::RwLock;

/// `fail(message)`: Aborts execution with an error message.
///
/// **Parameters**
/// - `message` (Any): The message to include in the error.
pub fn builtin_fail(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, NativeError> {
    Err(NativeError::runtime_error(format!(
        "Test failed explicitly: {:?}",
        args.first()
            .unwrap_or(&Value::String("no context provided".into()))
    )))
}
