use crate::ast::{Environment, Value};
use crate::interpreter::utils::{get_type_name, is_truthy};
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;

pub fn builtin_all(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "all() takes exactly one argument ({} given)",
            args.len()
        ));
    }

    let items = match &args[0] {
        Value::List(l) => l.borrow().clone(),
        Value::Tuple(t) => t.clone(),
        Value::Set(s) => s.borrow().iter().cloned().collect(),
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

    for item in items {
        if !is_truthy(&item) {
            return Ok(Value::Bool(false));
        }
    }
    Ok(Value::Bool(true))
}
