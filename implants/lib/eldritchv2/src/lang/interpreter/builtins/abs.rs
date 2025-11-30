use alloc::string::String;
use alloc::rc::Rc;
use core::cell::RefCell;
use crate::lang::ast::{Environment, Value};
use crate::lang::interpreter::utils::get_type_name;
use alloc::format;

pub fn builtin_abs(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("abs() takes exactly one argument ({} given)", args.len()));
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
