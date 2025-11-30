use alloc::string::String;
use alloc::rc::Rc;
use alloc::format;
use core::cell::RefCell;
use crate::lang::ast::{Environment, Value};
use crate::lang::interpreter::utils::is_truthy;

pub fn builtin_assert(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if !is_truthy(&args[0]) {
        return Err(format!(
            "Assertion failed: value '{:?}' is not truthy",
            args[0]
        ));
    }
    Ok(Value::None)
}
