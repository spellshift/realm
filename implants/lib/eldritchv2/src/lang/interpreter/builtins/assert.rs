use crate::lang::ast::{Environment, Value};
use crate::lang::interpreter::utils::is_truthy;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;

pub fn builtin_assert(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if !is_truthy(&args[0]) {
        return Err(format!(
            "Assertion failed: value '{:?}' is not truthy",
            args[0]
        ));
    }
    Ok(Value::None)
}
