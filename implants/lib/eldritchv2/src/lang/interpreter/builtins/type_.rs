use crate::lang::ast::{Environment, Value};
use crate::lang::interpreter::utils::get_type_name;
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;

pub fn builtin_type(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    Ok(Value::String(get_type_name(&args[0])))
}
