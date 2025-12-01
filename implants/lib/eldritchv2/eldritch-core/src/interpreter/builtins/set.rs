use crate::ast::{Environment, Value};
use crate::interpreter::utils::get_type_name;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use core::cell::RefCell;

pub fn builtin_set(_env: &Rc<RefCell<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::Set(Rc::new(RefCell::new(BTreeSet::new()))));
    }
    if args.len() != 1 {
        return Err(format!(
            "set() takes at most 1 argument ({} given)",
            args.len()
        ));
    }

    let items = match &args[0] {
        Value::List(l) => l.borrow().clone(),
        Value::Tuple(t) => t.clone(),
        Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
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

    let mut set = BTreeSet::new();
    for item in items {
        // Here we rely on Value implementing Ord.
        // Mutable types (List, Dict, Set) are compared by content in my implementation.
        // This deviates from Python (unhashable), but Eldritch V2 handles it this way for now.
        set.insert(item);
    }

    Ok(Value::Set(Rc::new(RefCell::new(set))))
}
