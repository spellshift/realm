use alloc::string::String;
use alloc::rc::Rc;
use core::cell::RefCell;
use crate::lang::ast::{Environment, Value};
use crate::lang::interpreter::utils::get_type_name;
use alloc::format;

pub fn builtin_max(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("max expected 1 argument, got 0".to_string());
    }
    if args.len() != 1 {
        // Technically max() supports multiple args: max(arg1, arg2, *args, key=func)
        // But prompt says: "max(x) returns the greatest element in the collection x"
        // I will implement single iterable argument version first.
        return Err(format!("max expected 1 argument, got {}", args.len()));
    }

    let items = match &args[0] {
        Value::List(l) => l.borrow().clone(),
        Value::Tuple(t) => t.clone(),
        Value::Set(s) => s.borrow().iter().cloned().collect(),
        Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
        Value::Dictionary(d) => d.borrow().keys().map(|k| Value::String(k.clone())).collect(),
        _ => return Err(format!("'{}' object is not iterable", get_type_name(&args[0]))),
    };

    if items.is_empty() {
        return Err("max() arg is an empty sequence".to_string());
    }

    let mut max_val = &items[0];
    for item in &items[1..] {
        if item > max_val {
            max_val = item;
        }
    }

    Ok(max_val.clone())
}
