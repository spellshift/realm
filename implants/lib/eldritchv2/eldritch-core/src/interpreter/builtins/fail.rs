use crate::ast::{Environment, Value};
use alloc::format;
use alloc::sync::Arc;
use alloc::string::String;
use spin::RwLock;

pub fn builtin_fail(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    Err(format!("Test failed explicitly: {}", args[0]))
}
