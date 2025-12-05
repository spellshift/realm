use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::sync::Arc;
use alloc::string::String;
use spin::RwLock;

pub fn builtin_type(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(String::from("type() expects exactly one argument"));
    }
    Ok(Value::String(get_type_name(&args[0])))
}
