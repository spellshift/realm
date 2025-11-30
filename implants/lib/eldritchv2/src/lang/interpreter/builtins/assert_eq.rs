use alloc::string::String;
use alloc::rc::Rc;
use alloc::format;
use core::cell::RefCell;
use crate::lang::ast::{Environment, Value};

pub fn builtin_assert_eq(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args[0] != args[1] {
        return Err(format!(
            "Assertion failed: left != right\n  Left:  {:?}\n  Right: {:?}",
            args[0], args[1]
        ));
    }
    Ok(Value::None)
}
