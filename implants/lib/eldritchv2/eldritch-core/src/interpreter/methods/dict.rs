use super::ArgCheck;
use crate::ast::Value;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use spin::RwLock;

pub fn handle_dict_methods(
    d: &Arc<RwLock<BTreeMap<Value, Value>>>,
    method: &str,
    args: &[Value],
) -> Option<Result<Value, alloc::string::String>> {
    match method {
        "keys" => Some((|| {
            args.require(0, "keys")?;
            let keys: Vec<Value> = d.read().keys().cloned().collect();
            Ok(Value::List(Arc::new(RwLock::new(keys))))
        })()),
        "values" => Some((|| {
            args.require(0, "values")?;
            let values: Vec<Value> = d.read().values().cloned().collect();
            Ok(Value::List(Arc::new(RwLock::new(values))))
        })()),
        "items" => Some((|| {
            args.require(0, "items")?;
            let items: Vec<Value> = d
                .read()
                .iter()
                .map(|(k, v)| Value::Tuple(vec![k.clone(), v.clone()]))
                .collect();
            Ok(Value::List(Arc::new(RwLock::new(items))))
        })()),
        "get" => Some((|| {
            args.require_range(1, 2, "get")?;
            let key = &args[0];
            let default = if args.len() == 2 {
                args[1].clone()
            } else {
                Value::None
            };
            match d.read().get(key) {
                Some(v) => Ok(v.clone()),
                None => Ok(default),
            }
        })()),
        "update" => Some((|| {
            args.require(1, "update")?;
            match &args[0] {
                Value::Dictionary(other) => {
                    let other_map = other.read().clone();
                    d.write().extend(other_map);
                    Ok(Value::None)
                }
                _ => Err("TypeError: update() requires a dictionary".into()),
            }
        })()),
        "popitem" => Some((|| {
            args.require(0, "popitem")?;
            let mut map = d.write();
            let last_key = map.keys().next_back().cloned();
            if let Some(k) = last_key {
                let v = map.remove(&k).unwrap();
                Ok(Value::Tuple(vec![k, v]))
            } else {
                Err("KeyError: popitem(): dictionary is empty".into())
            }
        })()),
        "clear" => Some((|| {
            args.require(0, "clear")?;
            d.write().clear();
            Ok(Value::None)
        })()),
        "pop" => Some((|| {
            args.require_range(1, 2, "pop")?;
            let key = &args[0];
            let mut map = d.write();
            if let Some(val) = map.remove(key) {
                return Ok(val);
            }
            if args.len() == 2 {
                Ok(args[1].clone())
            } else {
                Err(format!("KeyError: {}", key))
            }
        })()),
        "setdefault" => Some((|| {
            args.require_range(1, 2, "setdefault")?;
            let key = args[0].clone();
            let default = if args.len() == 2 {
                args[1].clone()
            } else {
                Value::None
            };
            let mut map = d.write();
            if let Some(val) = map.get(&key) {
                Ok(val.clone())
            } else {
                map.insert(key, default.clone());
                Ok(default)
            }
        })()),
        _ => None,
    }
}
