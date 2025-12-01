use crate::ast::{Environment, Value};
use crate::interpreter::utils::get_type_name;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use core::cell::RefCell;

pub fn builtin_sorted(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("sorted expected 1 argument, got 0".to_string());
    }
    if args.len() != 1 {
        // Ignoring key/reverse args for simplicity as per prompt description ("returns a new list... sorted order")
        return Err(format!("sorted expected 1 argument, got {}", args.len()));
    }

    let mut items = match &args[0] {
        Value::List(l) => l.borrow().clone(),
        Value::Tuple(t) => t.clone(),
        Value::Set(s) => s.borrow().iter().cloned().collect(),
        Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
        Value::Dictionary(d) => d
            .borrow()
            .keys()
            .map(|k| Value::String(k.clone()))
            .collect(),
        _ => {
            return Err(format!(
                "'{}' object is not iterable",
                get_type_name(&args[0])
            ))
        }
    };

    items.sort(); // uses Ord implementation on Value

    Ok(Value::List(Rc::new(RefCell::new(items))))
}
