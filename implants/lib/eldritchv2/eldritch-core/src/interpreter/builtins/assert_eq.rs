use crate::ast::{Environment, Value};
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;

pub fn builtin_assert_eq(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!(
            "assert_eq() takes exactly two arguments ({} given)",
            args.len()
        ));
    }
    if args[0] != args[1] {
        return Err(format!(
            "Assertion failed: left != right\n  Left:  {:?}\n  Right: {:?}",
            args[0], args[1]
        ));
    }
    Ok(Value::None)
}
