use alloc::string::String;
use alloc::rc::Rc;
use alloc::format;
use core::cell::RefCell;
use crate::lang::ast::{Environment, Value};

pub fn builtin_fail(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    Err(format!("Test failed explicitly: {}", args[0]))
}
