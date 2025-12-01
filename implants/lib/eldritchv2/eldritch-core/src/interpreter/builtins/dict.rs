use crate::ast::{Environment, Value};
use crate::interpreter::utils::get_type_name;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use core::cell::RefCell;

pub fn builtin_dict(
    _env: &Rc<RefCell<Environment>>,
    args: &[Value],
    kwargs: &BTreeMap<String, Value>,
) -> Result<Value, String> {
    if args.len() > 1 {
        return Err(format!(
            "dict expected at most 1 arguments, got {}",
            args.len()
        ));
    }

    let mut map = BTreeMap::new();

    // 1. Process positional argument (iterable of pairs)
    if let Some(iterable) = args.first() {
        match iterable {
            Value::Dictionary(d) => {
                // Copy other dict
                map = d.borrow().clone();
            }
            Value::List(l) => {
                let list = l.borrow();
                for (i, item) in list.iter().enumerate() {
                    process_pair(&mut map, item, i)?;
                }
            }
            Value::Tuple(t) => {
                for (i, item) in t.iter().enumerate() {
                    process_pair(&mut map, item, i)?;
                }
            }
            Value::Set(s) => {
                let set = s.borrow();
                for (i, item) in set.iter().enumerate() {
                    process_pair(&mut map, item, i)?;
                }
            }
            _ => {
                return Err(format!(
                    "'{}' object is not iterable",
                    get_type_name(iterable)
                ))
            }
        }
    }

    // 2. Process kwargs
    for (k, v) in kwargs {
        map.insert(k.clone(), v.clone());
    }

    Ok(Value::Dictionary(Rc::new(RefCell::new(map))))
}

fn process_pair(
    map: &mut BTreeMap<String, Value>,
    item: &Value,
    index: usize,
) -> Result<(), String> {
    match item {
        Value::List(l) => {
            let list = l.borrow();
            if list.len() != 2 {
                return Err(format!(
                    "dictionary update sequence element #{} has length {}; 2 is required",
                    index,
                    list.len()
                ));
            }
            let key = match &list[0] {
                Value::String(s) => s.clone(),
                _ => return Err("dict keys must be strings".to_string()),
            };
            map.insert(key, list[1].clone());
        }
        Value::Tuple(t) => {
            if t.len() != 2 {
                return Err(format!(
                    "dictionary update sequence element #{} has length {}; 2 is required",
                    index,
                    t.len()
                ));
            }
            let key = match &t[0] {
                Value::String(s) => s.clone(),
                _ => return Err("dict keys must be strings".to_string()),
            };
            map.insert(key, t[1].clone());
        }
        _ => {
            return Err(format!(
                "cannot convert dictionary update sequence element #{} to a sequence",
                index
            ))
        }
    }
    Ok(())
}
