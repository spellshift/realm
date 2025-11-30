use alloc::string::String;
use alloc::rc::Rc;
use alloc::format;
use core::cell::RefCell;
use crate::lang::ast::{Environment, Value};
use crate::lang::interpreter::utils::get_type_name;

pub fn builtin_int(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::Int(0));
    }
    match &args[0] {
        Value::Int(i) => Ok(Value::Int(*i)),
        Value::Float(f) => Ok(Value::Int(*f as i64)), // Truncate
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        Value::String(s) => s
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| format!("invalid literal for int(): '{}'", s)),
        _ => Err(format!(
            "int() argument must be a string, bytes or number, not '{}'",
            get_type_name(&args[0])
        )),
    }
}
