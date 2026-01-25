use super::get_all_builtins;
use crate::ast::{Environment, Value};
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

/// `builtins()`: Lists all built-in functions.
///
/// Returns a list of strings representing the names of all built-in functions
/// available in the global scope.
pub fn builtin_builtins(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("builtins() takes no arguments".to_string());
    }
    let mut names: Vec<String> = get_all_builtins()
        .into_iter()
        .map(|(n, _)| n.to_string())
        .collect();
    names.sort();
    let val_list: Vec<Value> = names.into_iter().map(Value::String).collect();
    Ok(Value::List(Arc::new(RwLock::new(val_list))))
}
