use crate::lang::ast::{Environment, Value};
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;

pub fn builtin_assert_eq(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args[0] != args[1] {
        return Err(format!(
            "Assertion failed: left != right\n  Left:  {:?}\n  Right: {:?}",
            args[0], args[1]
        ));
    }
    Ok(Value::None)
}
