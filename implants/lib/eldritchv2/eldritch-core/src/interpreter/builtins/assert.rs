use crate::ast::{Environment, Value};
use crate::interpreter::utils::is_truthy;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;

pub fn builtin_assert(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "assert() takes exactly one argument ({} given)",
            args.len()
        ));
    }
    if !is_truthy(&args[0]) {
        return Err(format!(
            "Assertion failed: value '{:?}' is not truthy",
            args[0]
        ));
    }
    Ok(Value::None)
}
