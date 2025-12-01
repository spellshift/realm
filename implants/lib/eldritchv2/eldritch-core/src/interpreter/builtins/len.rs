use crate::ast::{Environment, Value};
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;

pub fn builtin_len(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::Bytes(b) => Ok(Value::Int(b.len() as i64)),
        Value::List(l) => Ok(Value::Int(l.borrow().len() as i64)),
        Value::Dictionary(d) => Ok(Value::Int(d.borrow().len() as i64)),
        Value::Tuple(t) => Ok(Value::Int(t.len() as i64)),
        Value::Set(s) => Ok(Value::Int(s.borrow().len() as i64)),
        _ => Err(format!("'len()' is not defined for type: {:?}", args[0])),
    }
}
