use crate::ast::{Environment, Value};
use crate::interpreter::utils::get_type_name;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cell::RefCell;

pub fn builtin_zip(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::List(Rc::new(RefCell::new(Vec::new()))));
    }

    let mut iterators: Vec<Vec<Value>> = Vec::new();
    for arg in args {
        let items = match arg {
            Value::List(l) => l.borrow().clone(),
            Value::Tuple(t) => t.clone(),
            Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
            Value::Set(s) => s.borrow().iter().cloned().collect(),
            Value::Dictionary(d) => d
                .borrow()
                .keys()
                .map(|k| Value::String(k.clone()))
                .collect(),
            _ => return Err(format!("'{}' object is not iterable", get_type_name(arg))),
        };
        iterators.push(items);
    }

    let min_len = iterators.iter().map(|v| v.len()).min().unwrap_or(0);
    let mut result = Vec::with_capacity(min_len);

    for i in 0..min_len {
        let mut tuple_items = Vec::with_capacity(iterators.len());
        for iter in &iterators {
            tuple_items.push(iter[i].clone());
        }
        result.push(Value::Tuple(tuple_items));
    }

    Ok(Value::List(Rc::new(RefCell::new(result))))
}
