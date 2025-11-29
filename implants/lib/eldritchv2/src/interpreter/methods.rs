use super::utils::{compare_values, get_type_name};
use crate::ast::Value;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::cmp::Ordering;

pub fn call_bound_method(receiver: &Value, method: &str, args: &[Value]) -> Result<Value, String> {
    match (receiver, method) {
        (Value::List(l), "append") => {
            if args.len() != 1 {
                return Err("append() takes exactly one argument".into());
            }
            l.borrow_mut().push(args[0].clone());
            Ok(Value::None)
        }
        (Value::List(l), "extend") => {
            if args.len() != 1 {
                return Err("extend() takes exactly one argument".into());
            }
            let iterable = &args[0];
            match iterable {
                Value::List(other) => l.borrow_mut().extend(other.borrow().clone()),
                Value::Tuple(other) => l.borrow_mut().extend(other.clone()),
                _ => {
                    return Err(format!(
                        "extend() expects an iterable, got {}",
                        get_type_name(iterable)
                    ))
                }
            }
            Ok(Value::None)
        }
        (Value::List(l), "insert") => {
            if args.len() != 2 {
                return Err("insert() takes exactly two arguments".into());
            }
            let idx = match args[0] {
                Value::Int(i) => i,
                _ => return Err("insert() index must be an integer".into()),
            };
            let val = args[1].clone();
            let mut vec = l.borrow_mut();
            let len = vec.len() as i64;
            let index = if idx < 0 {
                (len + idx).max(0) as usize
            } else {
                idx.min(len) as usize
            };
            vec.insert(index, val);
            Ok(Value::None)
        }
        (Value::List(l), "remove") => {
            if args.len() != 1 {
                return Err("remove() takes exactly one argument".into());
            }
            let target = &args[0];
            let mut vec = l.borrow_mut();
            if let Some(pos) = vec.iter().position(|x| x == target) {
                vec.remove(pos);
                Ok(Value::None)
            } else {
                Err("ValueError: list.remove(x): x not in list".into())
            }
        }
        (Value::List(l), "index") => {
            if args.len() != 1 {
                return Err("index() takes exactly one argument".into());
            }
            let target = &args[0];
            let vec = l.borrow();
            if let Some(pos) = vec.iter().position(|x| x == target) {
                Ok(Value::Int(pos as i64))
            } else {
                Err("ValueError: list.index(x): x not in list".into())
            }
        }
        (Value::List(l), "pop") => {
            if let Some(v) = l.borrow_mut().pop() {
                Ok(v)
            } else {
                Err("pop from empty list".into())
            }
        }
        (Value::List(l), "sort") => {
            let mut vec = l.borrow_mut();
            vec.sort_by(|a, b| compare_values(a, b).unwrap_or(Ordering::Equal));
            Ok(Value::None)
        }

        (Value::Dictionary(d), "keys") => {
            let keys: Vec<Value> = d
                .borrow()
                .keys()
                .map(|k| Value::String(k.clone()))
                .collect();
            Ok(Value::List(Rc::new(RefCell::new(keys))))
        }
        (Value::Dictionary(d), "values") => {
            let values: Vec<Value> = d.borrow().values().cloned().collect();
            Ok(Value::List(Rc::new(RefCell::new(values))))
        }
        (Value::Dictionary(d), "items") => {
            let items: Vec<Value> = d
                .borrow()
                .iter()
                .map(|(k, v)| Value::Tuple(vec![Value::String(k.clone()), v.clone()]))
                .collect();
            Ok(Value::List(Rc::new(RefCell::new(items))))
        }
        (Value::Dictionary(d), "get") => {
            if args.len() < 1 || args.len() > 2 {
                return Err("get() takes 1 or 2 arguments".into());
            }
            let key = match &args[0] {
                Value::String(s) => s,
                v => return Err(format!("Dict keys must be strings, got {}", v)),
            };
            let default = if args.len() == 2 {
                args[1].clone()
            } else {
                Value::None
            };
            match d.borrow().get(key) {
                Some(v) => Ok(v.clone()),
                None => Ok(default),
            }
        }
        (Value::Dictionary(d), "update") => {
            if args.len() != 1 {
                return Err("update() takes exactly one argument".into());
            }
            match &args[0] {
                Value::Dictionary(other) => {
                    let other_map = other.borrow().clone();
                    d.borrow_mut().extend(other_map);
                    Ok(Value::None)
                }
                _ => Err("update() requires a dictionary".into()),
            }
        }
        (Value::Dictionary(d), "popitem") => {
            let mut map = d.borrow_mut();
            let last_key = map.keys().next_back().cloned();
            if let Some(k) = last_key {
                let v = map.remove(&k).unwrap();
                Ok(Value::Tuple(vec![Value::String(k), v]))
            } else {
                Err("popitem(): dictionary is empty".into())
            }
        }

        (Value::String(s), "split") => {
            let delim = if args.len() > 0 {
                args[0].to_string()
            } else {
                " ".to_string()
            };
            let parts: Vec<Value> = s
                .split(&delim)
                .map(|p| Value::String(p.to_string()))
                .collect();
            Ok(Value::List(Rc::new(RefCell::new(parts))))
        }
        (Value::String(s), "strip") => Ok(Value::String(s.trim().to_string())),
        (Value::String(s), "lower") => Ok(Value::String(s.to_lowercase())),
        (Value::String(s), "upper") => Ok(Value::String(s.to_uppercase())),
        (Value::String(s), "startswith") => {
            if args.len() != 1 {
                return Err("startswith() takes 1 argument".into());
            }
            let prefix = args[0].to_string();
            Ok(Value::Bool(s.starts_with(&prefix)))
        }
        (Value::String(s), "endswith") => {
            if args.len() != 1 {
                return Err("endswith() takes 1 argument".into());
            }
            let suffix = args[0].to_string();
            Ok(Value::Bool(s.ends_with(&suffix)))
        }
        (Value::String(s), "find") => {
            if args.len() != 1 {
                return Err("find() takes 1 argument".into());
            }
            let sub = args[0].to_string();
            match s.find(&sub) {
                Some(idx) => Ok(Value::Int(idx as i64)),
                None => Ok(Value::Int(-1)),
            }
        }
        (Value::String(s), "replace") => {
            if args.len() != 2 {
                return Err("replace() takes 2 arguments".into());
            }
            let old = args[0].to_string();
            let new = args[1].to_string();
            Ok(Value::String(s.replace(&old, &new)))
        }
        (Value::String(s), "join") => {
            if args.len() != 1 {
                return Err("join() takes 1 argument".into());
            }
            match &args[0] {
                Value::List(l) => {
                    let list = l.borrow();
                    let strs: Result<Vec<String>, _> = list
                        .iter()
                        .map(|v| match v {
                            Value::String(ss) => Ok(ss.clone()),
                            _ => Err("join() expects list of strings".to_string()),
                        })
                        .collect();
                    Ok(Value::String(strs?.join(s)))
                }
                _ => Err("join() expects a list".into()),
            }
        }
        (Value::String(s), "format") => {
            let mut result = String::new();
            let mut arg_idx = 0;
            let chars: Vec<char> = s.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                if chars[i] == '{' && i + 1 < chars.len() && chars[i + 1] == '}' {
                    if arg_idx >= args.len() {
                        return Err("tuple index out of range".into());
                    }
                    result.push_str(&args[arg_idx].to_string());
                    arg_idx += 1;
                    i += 2;
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
            }
            Ok(Value::String(result))
        }

        _ => Err(format!(
            "Object of type '{}' has no method '{}'",
            get_type_name(receiver),
            method
        )),
    }
}
