use alloc::string::String;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use crate::lang::ast::{Environment, Value};
use crate::lang::interpreter::utils::get_type_name;
use alloc::format;

pub fn builtin_list(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::List(Rc::new(RefCell::new(Vec::new()))));
    }
    if args.len() != 1 {
        return Err(format!("list() takes at most 1 argument ({} given)", args.len()));
    }

    let items = match &args[0] {
        Value::List(l) => l.borrow().clone(),
        Value::Tuple(t) => t.clone(),
        Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
        Value::Set(s) => s.borrow().iter().cloned().collect(),
        Value::Dictionary(d) => d.borrow().keys().map(|k| Value::String(k.clone())).collect(),
        _ => return Err(format!("'{}' object is not iterable", get_type_name(&args[0]))),
    };

    Ok(Value::List(Rc::new(RefCell::new(items))))
}
