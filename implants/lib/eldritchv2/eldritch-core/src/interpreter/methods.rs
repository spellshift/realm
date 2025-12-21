use super::super::ast::Value;
use super::introspection::{find_best_match, get_type_name, is_truthy};
use super::operations::{compare_values, values_equal};
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;
use spin::RwLock;

pub fn get_native_methods(value: &Value) -> Vec<String> {
    match value {
        Value::List(_) => vec![
            "append".to_string(),
            "extend".to_string(),
            "insert".to_string(),
            "remove".to_string(),
            "index".to_string(),
            "pop".to_string(),
            "sort".to_string(),
        ],
        Value::Dictionary(_) => vec![
            "keys".to_string(),
            "values".to_string(),
            "items".to_string(),
            "get".to_string(),
            "update".to_string(),
            "popitem".to_string(),
        ],
        Value::Set(_) => vec![
            "add".to_string(),
            "clear".to_string(),
            "contains".to_string(),
            "difference".to_string(),
            "discard".to_string(),
            "intersection".to_string(),
            "isdisjoint".to_string(),
            "issubset".to_string(),
            "issuperset".to_string(),
            "pop".to_string(),
            "remove".to_string(),
            "symmetric_difference".to_string(),
            "union".to_string(),
            "update".to_string(),
        ],
        Value::String(_) => vec![
            "split".to_string(),
            "splitlines".to_string(),
            "strip".to_string(),
            "lstrip".to_string(),
            "rstrip".to_string(),
            "lower".to_string(),
            "upper".to_string(),
            "capitalize".to_string(),
            "title".to_string(),
            "startswith".to_string(),
            "endswith".to_string(),
            "removeprefix".to_string(),
            "removesuffix".to_string(),
            "find".to_string(),
            "index".to_string(),
            "rfind".to_string(),
            "rindex".to_string(),
            "count".to_string(),
            "replace".to_string(),
            "join".to_string(),
            "format".to_string(),
            "partition".to_string(),
            "rpartition".to_string(),
            "rsplit".to_string(),
            "codepoints".to_string(),
            "elems".to_string(),
            "isalnum".to_string(),
            "isalpha".to_string(),
            "isdigit".to_string(),
            "islower".to_string(),
            "isupper".to_string(),
            "isspace".to_string(),
            "istitle".to_string(),
        ],
        _ => Vec::new(),
    }
}

// Helper to convert any iterable Value into a BTreeSet<Value> for set operations.
fn get_set_elements(v: &Value) -> Result<BTreeSet<Value>, String> {
    match v {
        Value::Set(s) => Ok(s.read().clone()),
        Value::List(l) => Ok(l.read().iter().cloned().collect()),
        Value::Tuple(t) => Ok(t.iter().cloned().collect()),
        Value::Dictionary(d) => Ok(d.read().keys().cloned().collect()),
        Value::String(s) => Ok(s.chars().map(|c| Value::String(c.to_string())).collect()),
        _ => Err(format!("'{}' object is not iterable", get_type_name(v))),
    }
}

pub fn call_bound_method(receiver: &Value, method: &str, args: &[Value]) -> Result<Value, String> {
    match (receiver, method) {
        (Value::List(l), "append") => {
            if args.len() != 1 {
                return Err("append() takes exactly one argument".into());
            }
            l.write().push(args[0].clone());
            Ok(Value::None)
        }
        (Value::List(l), "extend") => {
            if args.len() != 1 {
                return Err("extend() takes exactly one argument".into());
            }
            let iterable = &args[0];
            match iterable {
                Value::List(other) => {
                    // DEADLOCK FIX: Read other first, then write l.
                    // If l and other are the same list, l.write().extend(other.read().clone())
                    // would acquire the write lock on l, then try to acquire the read lock on other
                    // (which is l), causing a deadlock.
                    let items = other.read().clone();
                    l.write().extend(items);
                }
                Value::Tuple(other) => l.write().extend(other.clone()),
                _ => {
                    return Err(format!(
                        "extend() expects an iterable, got {}",
                        get_type_name(iterable)
                    ));
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
            let mut vec = l.write();
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
            let mut vec = l.write();
            if let Some(pos) = vec.iter().position(|x| values_equal(x, target)) {
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
            let vec = l.read();
            if let Some(pos) = vec.iter().position(|x| values_equal(x, target)) {
                Ok(Value::Int(pos as i64))
            } else {
                Err("ValueError: list.index(x): x not in list".into())
            }
        }
        (Value::List(l), "pop") => {
            if let Some(v) = l.write().pop() {
                Ok(v)
            } else {
                Err("pop from empty list".into())
            }
        }
        (Value::List(l), "sort") => {
            let mut vec = l.write();
            vec.sort_by(|a, b| compare_values(a, b).unwrap_or(Ordering::Equal));
            Ok(Value::None)
        }

        (Value::Dictionary(d), "keys") => {
            let keys: Vec<Value> = d.read().keys().cloned().collect();
            Ok(Value::List(Arc::new(RwLock::new(keys))))
        }
        (Value::Dictionary(d), "values") => {
            let values: Vec<Value> = d.read().values().cloned().collect();
            Ok(Value::List(Arc::new(RwLock::new(values))))
        }
        (Value::Dictionary(d), "items") => {
            let items: Vec<Value> = d
                .read()
                .iter()
                .map(|(k, v)| Value::Tuple(vec![k.clone(), v.clone()]))
                .collect();
            Ok(Value::List(Arc::new(RwLock::new(items))))
        }
        (Value::Dictionary(d), "get") => {
            if args.is_empty() || args.len() > 2 {
                return Err("get() takes 1 or 2 arguments".into());
            }
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
        }
        (Value::Dictionary(d), "update") => {
            if args.len() != 1 {
                return Err("update() takes exactly one argument".into());
            }
            match &args[0] {
                Value::Dictionary(other) => {
                    let other_map = other.read().clone();
                    d.write().extend(other_map);
                    Ok(Value::None)
                }
                _ => Err("update() requires a dictionary".into()),
            }
        }
        (Value::Dictionary(d), "popitem") => {
            let mut map = d.write();
            let last_key = map.keys().next_back().cloned();
            if let Some(k) = last_key {
                let v = map.remove(&k).unwrap();
                Ok(Value::Tuple(vec![k, v]))
            } else {
                Err("popitem(): dictionary is empty".into())
            }
        }

        (Value::Set(s), "add") => {
            if args.len() != 1 {
                return Err("add() takes exactly one argument".into());
            }
            s.write().insert(args[0].clone());
            Ok(Value::None)
        }
        (Value::Set(s), "clear") => {
            s.write().clear();
            Ok(Value::None)
        }
        (Value::Set(s), "contains") => {
            if args.len() != 1 {
                return Err("contains() takes exactly one argument".into());
            }
            Ok(Value::Bool(s.read().contains(&args[0])))
        }
        (Value::Set(s), "difference") => {
            if args.len() != 1 {
                return Err("difference() takes exactly one argument".into());
            }
            let other_set = get_set_elements(&args[0])?;
            let diff: BTreeSet<Value> = s.read().difference(&other_set).cloned().collect();
            Ok(Value::Set(Arc::new(RwLock::new(diff))))
        }
        (Value::Set(s), "discard") => {
            if args.len() != 1 {
                return Err("discard() takes exactly one argument".into());
            }
            s.write().remove(&args[0]);
            Ok(Value::None)
        }
        (Value::Set(s), "intersection") => {
            if args.len() != 1 {
                return Err("intersection() takes exactly one argument".into());
            }
            let other_set = get_set_elements(&args[0])?;
            let inter: BTreeSet<Value> = s.read().intersection(&other_set).cloned().collect();
            Ok(Value::Set(Arc::new(RwLock::new(inter))))
        }
        (Value::Set(s), "isdisjoint") => {
            if args.len() != 1 {
                return Err("isdisjoint() takes exactly one argument".into());
            }
            let other_set = get_set_elements(&args[0])?;
            Ok(Value::Bool(s.read().is_disjoint(&other_set)))
        }
        (Value::Set(s), "issubset") => {
            if args.len() != 1 {
                return Err("issubset() takes exactly one argument".into());
            }
            let other_set = get_set_elements(&args[0])?;
            Ok(Value::Bool(s.read().is_subset(&other_set)))
        }
        (Value::Set(s), "issuperset") => {
            if args.len() != 1 {
                return Err("issuperset() takes exactly one argument".into());
            }
            let other_set = get_set_elements(&args[0])?;
            Ok(Value::Bool(s.read().is_superset(&other_set)))
        }
        (Value::Set(s), "pop") => {
            // Remove the LAST element, per user request (and consistent with list.pop())
            // BTreeSet is ordered, so this removes the largest element.
            let mut set = s.write();
            if set.is_empty() {
                return Err("pop from empty set".into());
            }
            // Use iterator to get last element without nightly features
            let last = set.iter().next_back().cloned();
            if let Some(v) = last {
                set.remove(&v);
                Ok(v)
            } else {
                Err("pop from empty set".into())
            }
        }
        (Value::Set(s), "remove") => {
            if args.len() != 1 {
                return Err("remove() takes exactly one argument".into());
            }
            if !s.write().remove(&args[0]) {
                return Err(format!("KeyError: {}", args[0]));
            }
            Ok(Value::None)
        }
        (Value::Set(s), "symmetric_difference") => {
            if args.len() != 1 {
                return Err("symmetric_difference() takes exactly one argument".into());
            }
            let other_set = get_set_elements(&args[0])?;
            let sym: BTreeSet<Value> = s.read().symmetric_difference(&other_set).cloned().collect();
            Ok(Value::Set(Arc::new(RwLock::new(sym))))
        }
        (Value::Set(s), "union") => {
            if args.len() != 1 {
                return Err("union() takes exactly one argument".into());
            }
            let other_set = get_set_elements(&args[0])?;
            let u: BTreeSet<Value> = s.read().union(&other_set).cloned().collect();
            Ok(Value::Set(Arc::new(RwLock::new(u))))
        }
        (Value::Set(s), "update") => {
            if args.len() != 1 {
                return Err("update() takes exactly one argument".into());
            }
            let other_set = get_set_elements(&args[0])?;
            s.write().extend(other_set);
            Ok(Value::None)
        }

        (Value::String(s), "split") => {
            let parts: Vec<Value> = if args.is_empty() {
                // Default split: split by whitespace (runs of whitespace are one separator)
                s.split_whitespace()
                    .map(|p| Value::String(p.to_string()))
                    .collect()
            } else {
                // Split by specific delimiter
                let delim = args[0].to_string();
                s.split(&delim)
                    .map(|p| Value::String(p.to_string()))
                    .collect()
            };
            Ok(Value::List(Arc::new(RwLock::new(parts))))
        }
        (Value::String(s), "splitlines") => {
            let keepends = if !args.is_empty() {
                is_truthy(&args[0])
            } else {
                false
            };
            let lines: Vec<Value> = if keepends {
                // Not perfectly matching python's splitlines(keepends=True) split behavior on all boundaries, but roughly
                s.split_inclusive('\n')
                    .map(|p| Value::String(p.to_string()))
                    .collect()
            } else {
                s.lines().map(|p| Value::String(p.to_string())).collect()
            };
            Ok(Value::List(Arc::new(RwLock::new(lines))))
        }
        (Value::String(s), "strip") => Ok(Value::String(s.trim().to_string())),
        (Value::String(s), "lstrip") => Ok(Value::String(s.trim_start().to_string())),
        (Value::String(s), "rstrip") => Ok(Value::String(s.trim_end().to_string())),
        (Value::String(s), "lower") => Ok(Value::String(s.to_lowercase())),
        (Value::String(s), "upper") => Ok(Value::String(s.to_uppercase())),
        (Value::String(s), "capitalize") => {
            let mut c = s.chars();
            match c.next() {
                None => Ok(Value::String(String::new())),
                Some(f) => {
                    let res = f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase();
                    Ok(Value::String(res))
                }
            }
        }
        (Value::String(s), "title") => {
            // Simplified title case: capitalize first letter of each word
            // We removed the unused _res variable
            let mut result = String::new();
            let mut cap_next = true;
            for c in s.chars() {
                if c.is_alphabetic() {
                    if cap_next {
                        result.extend(c.to_uppercase());
                        cap_next = false;
                    } else {
                        result.extend(c.to_lowercase());
                    }
                } else {
                    result.push(c);
                    cap_next = true;
                }
            }
            Ok(Value::String(result))
        }
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
        (Value::String(s), "removeprefix") => {
            if args.len() != 1 {
                return Err("removeprefix() takes 1 argument".into());
            }
            let prefix = args[0].to_string();
            if s.starts_with(&prefix) {
                Ok(Value::String(s[prefix.len()..].to_string()))
            } else {
                Ok(Value::String(s.clone()))
            }
        }
        (Value::String(s), "removesuffix") => {
            if args.len() != 1 {
                return Err("removesuffix() takes 1 argument".into());
            }
            let suffix = args[0].to_string();
            if s.ends_with(&suffix) {
                Ok(Value::String(s[..s.len() - suffix.len()].to_string()))
            } else {
                Ok(Value::String(s.clone()))
            }
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
        (Value::String(s), "index") => {
            if args.len() != 1 {
                return Err("index() takes 1 argument".into());
            }
            let sub = args[0].to_string();
            match s.find(&sub) {
                Some(idx) => Ok(Value::Int(idx as i64)),
                None => Err("ValueError: substring not found".into()),
            }
        }
        (Value::String(s), "rfind") => {
            if args.len() != 1 {
                return Err("rfind() takes 1 argument".into());
            }
            let sub = args[0].to_string();
            match s.rfind(&sub) {
                Some(idx) => Ok(Value::Int(idx as i64)),
                None => Ok(Value::Int(-1)),
            }
        }
        (Value::String(s), "rindex") => {
            if args.len() != 1 {
                return Err("rindex() takes 1 argument".into());
            }
            let sub = args[0].to_string();
            match s.rfind(&sub) {
                Some(idx) => Ok(Value::Int(idx as i64)),
                None => Err("ValueError: substring not found".into()),
            }
        }
        (Value::String(s), "count") => {
            if args.len() != 1 {
                return Err("count() takes 1 argument".into());
            }
            let sub = args[0].to_string();
            if sub.is_empty() {
                return Ok(Value::Int((s.len() + 1) as i64));
            }
            let count = s.matches(&sub).count();
            Ok(Value::Int(count as i64))
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
                    let list = l.read();
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
        (Value::String(s), "partition") => {
            if args.len() != 1 {
                return Err("partition() takes 1 argument".into());
            }
            let sep = args[0].to_string();
            if let Some(idx) = s.find(&sep) {
                let sep_len = sep.len(); // Clone logic handled by creating strings below
                Ok(Value::Tuple(vec![
                    Value::String(s[..idx].to_string()),
                    Value::String(sep),
                    Value::String(s[idx + sep_len..].to_string()),
                ]))
            } else {
                Ok(Value::Tuple(vec![
                    Value::String(s.clone()),
                    Value::String("".to_string()),
                    Value::String("".to_string()),
                ]))
            }
        }
        (Value::String(s), "rpartition") => {
            if args.len() != 1 {
                return Err("rpartition() takes 1 argument".into());
            }
            let sep = args[0].to_string();
            if let Some(idx) = s.rfind(&sep) {
                let sep_len = sep.len();
                Ok(Value::Tuple(vec![
                    Value::String(s[..idx].to_string()),
                    Value::String(sep),
                    Value::String(s[idx + sep_len..].to_string()),
                ]))
            } else {
                Ok(Value::Tuple(vec![
                    Value::String("".to_string()),
                    Value::String("".to_string()),
                    Value::String(s.clone()),
                ]))
            }
        }
        (Value::String(s), "rsplit") => {
            let delim = if !args.is_empty() {
                args[0].to_string()
            } else {
                " ".to_string()
            };
            // Note: Rust's rsplit is an iterator that yields from end to start.
            // Python's rsplit returns list in forward order.
            let mut parts: Vec<Value> = s
                .rsplit(&delim)
                .map(|p| Value::String(p.to_string()))
                .collect();
            parts.reverse();
            Ok(Value::List(Arc::new(RwLock::new(parts))))
        }
        (Value::String(s), "codepoints") => {
            let points: Vec<Value> = s.chars().map(|c| Value::Int(c as i64)).collect();
            Ok(Value::List(Arc::new(RwLock::new(points))))
        }
        (Value::String(s), "elems") => {
            let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
            Ok(Value::List(Arc::new(RwLock::new(chars))))
        }
        (Value::String(s), "isalnum") => Ok(Value::Bool(
            !s.is_empty() && s.chars().all(|c| c.is_alphanumeric()),
        )),
        (Value::String(s), "isalpha") => Ok(Value::Bool(
            !s.is_empty() && s.chars().all(|c| c.is_alphabetic()),
        )),
        (Value::String(s), "isdigit") => Ok(Value::Bool(
            !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()),
        )), // Python isdigit is unicode digits, but ascii is safer bet for now
        (Value::String(s), "islower") => Ok(Value::Bool(
            !s.is_empty() && s.chars().any(|c| c.is_alphabetic()) && s == &s.to_lowercase(),
        )),
        (Value::String(s), "isupper") => Ok(Value::Bool(
            !s.is_empty() && s.chars().any(|c| c.is_alphabetic()) && s == &s.to_uppercase(),
        )),
        (Value::String(s), "isspace") => Ok(Value::Bool(
            !s.is_empty() && s.chars().all(|c| c.is_whitespace()),
        )),
        (Value::String(s), "istitle") => {
            if s.is_empty() {
                return Ok(Value::Bool(false));
            }
            let mut cased = false;
            let mut _prev_cased = false;
            let mut expected_upper = true;
            for c in s.chars() {
                if c.is_uppercase() {
                    if !expected_upper {
                        return Ok(Value::Bool(false));
                    }
                    expected_upper = false;
                    cased = true;
                    _prev_cased = true;
                } else if c.is_lowercase() {
                    if expected_upper {
                        return Ok(Value::Bool(false));
                    }
                    cased = true;
                    _prev_cased = true;
                } else {
                    expected_upper = true;
                    _prev_cased = false;
                }
            }
            Ok(Value::Bool(cased))
        }

        _ => {
            let mut msg = format!(
                "Object of type '{}' has no method '{}'",
                get_type_name(receiver),
                method
            );
            // Suggest similar methods
            let candidates = get_native_methods(receiver);
            if let Some(suggestion) = find_best_match(method, &candidates) {
                msg.push_str(&format!("\nDid you mean '{suggestion}'?"));
            }
            Err(msg)
        }
    }
}
