use alloc::string::String;
use alloc::rc::Rc;
use core::cell::RefCell;
use crate::lang::ast::{Environment, Value};
use crate::lang::interpreter::utils::is_truthy;

pub fn builtin_bool(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    Ok(Value::Bool(is_truthy(&args[0])))
}
