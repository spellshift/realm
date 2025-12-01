use crate::lang::ast::{Environment, Value};
use crate::lang::interpreter::utils::get_type_name;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;

pub fn builtin_enumerate(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    let iterable = &args[0];
    let start = if args.len() > 1 {
        match args[1] {
            Value::Int(i) => i,
            _ => return Err("enumerate() start must be an integer".to_string()),
        }
    } else {
        0
    };
    let items = match iterable {
        Value::List(l) => l.borrow().clone(),
        Value::Tuple(t) => t.clone(),
        Value::Set(s) => s.borrow().iter().cloned().collect(),
        Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
        _ => {
            return Err(format!(
                "Type '{:?}' is not iterable",
                get_type_name(iterable)
            ))
        }
    };
    let mut pairs = Vec::new();
    for (i, item) in items.into_iter().enumerate() {
        pairs.push(Value::Tuple(vec![Value::Int(i as i64 + start), item]));
    }
    Ok(Value::List(Rc::new(RefCell::new(pairs))))
}
