use alloc::string::String;
use alloc::rc::Rc;
use core::cell::RefCell;
use crate::lang::ast::{Environment, Value};
use crate::lang::interpreter::utils::get_type_name;

pub fn builtin_type(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    Ok(Value::String(get_type_name(&args[0])))
}
