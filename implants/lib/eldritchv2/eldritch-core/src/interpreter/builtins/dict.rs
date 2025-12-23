use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use spin::RwLock;

/// `dict(**kwargs)` or `dict(iterable, **kwargs)`: Creates a dictionary.
///
/// **Parameters**
/// - `iterable` (Iterable): An iterable of key-value pairs (tuples/lists of length 2).
/// - `**kwargs` (Any): Keyword arguments to add to the dictionary.
pub fn builtin_dict(
    _env: &Arc<RwLock<Environment>>,
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
                map = d.read().clone();
            }
            Value::List(l) => {
                let list = l.read();
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
                let set = s.read();
                for (i, item) in set.iter().enumerate() {
                    process_pair(&mut map, item, i)?;
                }
            }
            _ => {
                return Err(format!(
                    "'{}' object is not iterable",
                    get_type_name(iterable)
                ));
            }
        }
    }

    // 2. Process kwargs
    for (k, v) in kwargs {
        map.insert(Value::String(k.clone()), v.clone());
    }

    Ok(Value::Dictionary(Arc::new(RwLock::new(map))))
}

fn process_pair(
    map: &mut BTreeMap<Value, Value>,
    item: &Value,
    index: usize,
) -> Result<(), String> {
    match item {
        Value::List(l) => {
            let list = l.read();
            if list.len() != 2 {
                return Err(format!(
                    "dictionary update sequence element #{} has length {}; 2 is required",
                    index,
                    list.len()
                ));
            }
            let key = list[0].clone();
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
            let key = t[0].clone();
            map.insert(key, t[1].clone());
        }
        _ => {
            return Err(format!(
                "cannot convert dictionary update sequence element #{index} to a sequence"
            ));
        }
    }
    Ok(())
}
