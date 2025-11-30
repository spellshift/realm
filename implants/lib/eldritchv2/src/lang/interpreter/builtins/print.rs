use alloc::string::String;
use alloc::rc::Rc;
use core::cell::RefCell;
use crate::lang::ast::{Environment, Value};

pub fn builtin_print(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    #[cfg(feature = "std")]
    {
        println!("{}", args[0]);
    }
    #[cfg(not(feature = "std"))]
    let _ = args;
    Ok(Value::None)
}
