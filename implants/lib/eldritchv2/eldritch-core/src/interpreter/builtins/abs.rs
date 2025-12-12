use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::format;
use alloc::sync::Arc;
use alloc::string::String;
use spin::RwLock;

pub fn builtin_abs(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "abs() takes exactly one argument ({} given)",
            args.len()
        ));
    }
    match &args[0] {
        Value::Int(i) => Ok(Value::Int(i.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err(format!(
            "bad operand type for abs(): '{}'",
            get_type_name(&args[0])
        )),
    }
}
