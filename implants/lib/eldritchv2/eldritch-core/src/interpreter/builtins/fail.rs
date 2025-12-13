use crate::ast::{Environment, Value};
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use spin::RwLock;

/// `fail(message)`: Aborts execution with an error message.
///
/// **Parameters**
/// - `message` (Any): The message to include in the error.
pub fn builtin_fail(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    Err(format!("Test failed explicitly: {}", args[0]))
}
