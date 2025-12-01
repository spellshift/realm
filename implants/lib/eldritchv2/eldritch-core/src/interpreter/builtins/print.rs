use crate::lang::ast::{Environment, Value};
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;

pub fn builtin_print(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    #[cfg(feature = "std")]
    {
        println!("{}", args[0]);
    }
    #[cfg(not(feature = "std"))]
    let _ = args;
    Ok(Value::None)
}
