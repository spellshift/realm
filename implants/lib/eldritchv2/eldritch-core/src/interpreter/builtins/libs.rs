use crate::ast::{Environment, Value};
use crate::global_libs::get_global_libraries;
use alloc::sync::Arc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use spin::RwLock;

pub fn builtin_libs(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("libs() takes no arguments".to_string());
    }
    let libs = get_global_libraries();
    let mut names: Vec<String> = libs.keys().cloned().collect();
    names.sort();
    let val_list: Vec<Value> = names.into_iter().map(Value::String).collect();
    Ok(Value::List(Arc::new(RwLock::new(val_list))))
}
