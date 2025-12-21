use super::super::ast::Value;
use super::introspection::{find_best_match, get_type_name, is_truthy};
use super::operations::{compare_values, values_equal};
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;
use spin::RwLock;

// --- Argument Validation Helper ---

trait ArgCheck {
    fn require(&self, count: usize, name: &str) -> Result<(), String>;
    fn require_range(&self, min: usize, max: usize, name: &str) -> Result<(), String>;
}

impl ArgCheck for [Value] {
    fn require(&self, count: usize, name: &str) -> Result<(), String> {
        if self.len() != count {
            return Err(format!(
                "TypeError: {}() takes exactly {} argument{}",
                name,
                count,
                if count != 1 { "s" } else { "" }
            ));
        }
        Ok(())
    }

    fn require_range(&self, min: usize, max: usize, name: &str) -> Result<(), String> {
        if self.len() < min || self.len() > max {
            return Err(format!(
                "TypeError: {}() takes between {} and {} arguments",
                name, min, max
            ));
        }
        Ok(())
    }
}

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
        _ => Err(format!(
            "TypeError: '{}' object is not iterable",
            get_type_name(v)
        )),
    }
}

// --- Specific Handlers ---

fn handle_list_methods(
    l: &Arc<RwLock<Vec<Value>>>,
    method: &str,
    args: &[Value],
) -> Option<Result<Value, String>> {
    match method {
        "append" => Some((|| {
            args.require(1, "append")?;
            l.write().push(args[0].clone());
            Ok(Value::None)
        })()),
        "extend" => Some((|| {
            args.require(1, "extend")?;
            let iterable = &args[0];
            match iterable {
                Value::List(other) => {
                    // DEADLOCK FIX: Read other first, then write l.
                    let items = other.read().clone();
                    l.write().extend(items);
                }
                Value::Tuple(other) => l.write().extend(other.clone()),
                _ => {
                    return Err(format!(
                        "TypeError: extend() expects an iterable, got {}",
                        get_type_name(iterable)
                    ));
                }
            }
            Ok(Value::None)
        })()),
        "insert" => Some((|| {
            args.require(2, "insert")?;
            let idx = match args[0] {
                Value::Int(i) => i,
                _ => return Err("TypeError: insert() index must be an integer".into()),
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
        })()),
        "remove" => Some((|| {
            args.require(1, "remove")?;
            let target = &args[0];
            let mut vec = l.write();
            if let Some(pos) = vec.iter().position(|x| values_equal(x, target)) {
                vec.remove(pos);
                Ok(Value::None)
            } else {
                Err("ValueError: list.remove(x): x not in list".into())
            }
        })()),
        "index" => Some((|| {
            args.require(1, "index")?;
            let target = &args[0];
            let vec = l.read();
            if let Some(pos) = vec.iter().position(|x| values_equal(x, target)) {
                Ok(Value::Int(pos as i64))
            } else {
                Err("ValueError: list.index(x): x not in list".into())
            }
        })()),
        "pop" => Some((|| {
            args.require(0, "pop")?;
            if let Some(v) = l.write().pop() {
                Ok(v)
            } else {
                Err("IndexError: pop from empty list".into())
            }
        })()),
        "sort" => Some((|| {
            args.require(0, "sort")?;
            let mut vec = l.write();
            vec.sort_by(|a, b| compare_values(a, b).unwrap_or(Ordering::Equal));
            Ok(Value::None)
        })()),
        _ => None,
    }
}

fn handle_dict_methods(
    d: &Arc<RwLock<BTreeMap<Value, Value>>>,
    method: &str,
    args: &[Value],
) -> Option<Result<Value, String>> {
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
        _ => None,
    }
}

fn handle_set_methods(
    s: &Arc<RwLock<BTreeSet<Value>>>,
    method: &str,
    args: &[Value],
) -> Option<Result<Value, String>> {
    match method {
        "add" => Some((|| {
            args.require(1, "add")?;
            s.write().insert(args[0].clone());
            Ok(Value::None)
        })()),
        "clear" => Some((|| {
            args.require(0, "clear")?;
            s.write().clear();
            Ok(Value::None)
        })()),
        "contains" => Some((|| {
            args.require(1, "contains")?;
            Ok(Value::Bool(s.read().contains(&args[0])))
        })()),
        "difference" => Some((|| {
            args.require(1, "difference")?;
            let other_set = get_set_elements(&args[0])?;
            let diff: BTreeSet<Value> = s.read().difference(&other_set).cloned().collect();
            Ok(Value::Set(Arc::new(RwLock::new(diff))))
        })()),
        "discard" => Some((|| {
            args.require(1, "discard")?;
            s.write().remove(&args[0]);
            Ok(Value::None)
        })()),
        "intersection" => Some((|| {
            args.require(1, "intersection")?;
            let other_set = get_set_elements(&args[0])?;
            let inter: BTreeSet<Value> = s.read().intersection(&other_set).cloned().collect();
            Ok(Value::Set(Arc::new(RwLock::new(inter))))
        })()),
        "isdisjoint" => Some((|| {
            args.require(1, "isdisjoint")?;
            let other_set = get_set_elements(&args[0])?;
            Ok(Value::Bool(s.read().is_disjoint(&other_set)))
        })()),
        "issubset" => Some((|| {
            args.require(1, "issubset")?;
            let other_set = get_set_elements(&args[0])?;
            Ok(Value::Bool(s.read().is_subset(&other_set)))
        })()),
        "issuperset" => Some((|| {
            args.require(1, "issuperset")?;
            let other_set = get_set_elements(&args[0])?;
            Ok(Value::Bool(s.read().is_superset(&other_set)))
        })()),
        "pop" => Some((|| {
            args.require(0, "pop")?;
            let mut set = s.write();
            if set.is_empty() {
                return Err("KeyError: pop from empty set".into());
            }
            let last = set.iter().next_back().cloned();
            if let Some(v) = last {
                set.remove(&v);
                Ok(v)
            } else {
                Err("KeyError: pop from empty set".into())
            }
        })()),
        "remove" => Some((|| {
            args.require(1, "remove")?;
            if !s.write().remove(&args[0]) {
                return Err(format!("KeyError: {}", args[0]));
            }
            Ok(Value::None)
        })()),
        "symmetric_difference" => Some((|| {
            args.require(1, "symmetric_difference")?;
            let other_set = get_set_elements(&args[0])?;
            let sym: BTreeSet<Value> = s.read().symmetric_difference(&other_set).cloned().collect();
            Ok(Value::Set(Arc::new(RwLock::new(sym))))
        })()),
        "union" => Some((|| {
            args.require(1, "union")?;
            let other_set = get_set_elements(&args[0])?;
            let u: BTreeSet<Value> = s.read().union(&other_set).cloned().collect();
            Ok(Value::Set(Arc::new(RwLock::new(u))))
        })()),
        "update" => Some((|| {
            args.require(1, "update")?;
            let other_set = get_set_elements(&args[0])?;
            s.write().extend(other_set);
            Ok(Value::None)
        })()),
        _ => None,
    }
}

fn handle_string_methods(s: &str, method: &str, args: &[Value]) -> Option<Result<Value, String>> {
    match method {
        "split" => Some((|| {
            let parts: Vec<Value> = if args.is_empty() {
                s.split_whitespace()
                    .map(|p| Value::String(p.to_string()))
                    .collect()
            } else {
                let delim = args[0].to_string();
                s.split(&delim)
                    .map(|p| Value::String(p.to_string()))
                    .collect()
            };
            Ok(Value::List(Arc::new(RwLock::new(parts))))
        })()),
        "splitlines" => Some((|| {
            let keepends = if !args.is_empty() {
                is_truthy(&args[0])
            } else {
                false
            };
            let lines: Vec<Value> = if keepends {
                s.split_inclusive('\n')
                    .map(|p| Value::String(p.to_string()))
                    .collect()
            } else {
                s.lines().map(|p| Value::String(p.to_string())).collect()
            };
            Ok(Value::List(Arc::new(RwLock::new(lines))))
        })()),
        "strip" => Some((|| {
            args.require(0, "strip")?;
            Ok(Value::String(s.trim().to_string()))
        })()),
        "lstrip" => Some((|| {
            args.require(0, "lstrip")?;
            Ok(Value::String(s.trim_start().to_string()))
        })()),
        "rstrip" => Some((|| {
            args.require(0, "rstrip")?;
            Ok(Value::String(s.trim_end().to_string()))
        })()),
        "lower" => Some((|| {
            args.require(0, "lower")?;
            Ok(Value::String(s.to_lowercase()))
        })()),
        "upper" => Some((|| {
            args.require(0, "upper")?;
            Ok(Value::String(s.to_uppercase()))
        })()),
        "capitalize" => Some((|| {
            args.require(0, "capitalize")?;
            let mut c = s.chars();
            match c.next() {
                None => Ok(Value::String(String::new())),
                Some(f) => {
                    let res = f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase();
                    Ok(Value::String(res))
                }
            }
        })()),
        "title" => Some((|| {
            args.require(0, "title")?;
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
        })()),
        "startswith" => Some((|| {
            args.require(1, "startswith")?;
            let prefix = args[0].to_string();
            Ok(Value::Bool(s.starts_with(&prefix)))
        })()),
        "endswith" => Some((|| {
            args.require(1, "endswith")?;
            let suffix = args[0].to_string();
            Ok(Value::Bool(s.ends_with(&suffix)))
        })()),
        "removeprefix" => Some((|| {
            args.require(1, "removeprefix")?;
            let prefix = args[0].to_string();
            if s.starts_with(&prefix) {
                Ok(Value::String(s[prefix.len()..].to_string()))
            } else {
                Ok(Value::String(s.to_string()))
            }
        })()),
        "removesuffix" => Some((|| {
            args.require(1, "removesuffix")?;
            let suffix = args[0].to_string();
            if s.ends_with(&suffix) {
                Ok(Value::String(s[..s.len() - suffix.len()].to_string()))
            } else {
                Ok(Value::String(s.to_string()))
            }
        })()),
        "find" => Some((|| {
            args.require(1, "find")?;
            let sub = args[0].to_string();
            match s.find(&sub) {
                Some(idx) => Ok(Value::Int(idx as i64)),
                None => Ok(Value::Int(-1)),
            }
        })()),
        "index" => Some((|| {
            args.require(1, "index")?;
            let sub = args[0].to_string();
            match s.find(&sub) {
                Some(idx) => Ok(Value::Int(idx as i64)),
                None => Err("ValueError: substring not found".into()),
            }
        })()),
        "rfind" => Some((|| {
            args.require(1, "rfind")?;
            let sub = args[0].to_string();
            match s.rfind(&sub) {
                Some(idx) => Ok(Value::Int(idx as i64)),
                None => Ok(Value::Int(-1)),
            }
        })()),
        "rindex" => Some((|| {
            args.require(1, "rindex")?;
            let sub = args[0].to_string();
            match s.rfind(&sub) {
                Some(idx) => Ok(Value::Int(idx as i64)),
                None => Err("ValueError: substring not found".into()),
            }
        })()),
        "count" => Some((|| {
            args.require(1, "count")?;
            let sub = args[0].to_string();
            if sub.is_empty() {
                return Ok(Value::Int((s.len() + 1) as i64));
            }
            let count = s.matches(&sub).count();
            Ok(Value::Int(count as i64))
        })()),
        "replace" => Some((|| {
            args.require(2, "replace")?;
            let old = args[0].to_string();
            let new = args[1].to_string();
            Ok(Value::String(s.replace(&old, &new)))
        })()),
        "join" => Some((|| {
            args.require(1, "join")?;
            match &args[0] {
                Value::List(l) => {
                    let list = l.read();
                    let strs: Result<Vec<String>, _> = list
                        .iter()
                        .map(|v| match v {
                            Value::String(ss) => Ok(ss.clone()),
                            _ => Err("TypeError: join() expects list of strings".to_string()),
                        })
                        .collect();
                    Ok(Value::String(strs?.join(s)))
                }
                _ => Err("TypeError: join() expects a list".into()),
            }
        })()),
        "format" => Some((|| {
            let mut result = String::new();
            let mut arg_idx = 0;
            let chars: Vec<char> = s.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                if chars[i] == '{' && i + 1 < chars.len() && chars[i + 1] == '}' {
                    if arg_idx >= args.len() {
                        return Err("IndexError: tuple index out of range".into());
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
        })()),
        "partition" => Some((|| {
            args.require(1, "partition")?;
            let sep = args[0].to_string();
            if let Some(idx) = s.find(&sep) {
                let sep_len = sep.len();
                Ok(Value::Tuple(vec![
                    Value::String(s[..idx].to_string()),
                    Value::String(sep),
                    Value::String(s[idx + sep_len..].to_string()),
                ]))
            } else {
                Ok(Value::Tuple(vec![
                    Value::String(s.to_string()),
                    Value::String("".to_string()),
                    Value::String("".to_string()),
                ]))
            }
        })()),
        "rpartition" => Some((|| {
            args.require(1, "rpartition")?;
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
                    Value::String(s.to_string()),
                ]))
            }
        })()),
        "rsplit" => Some((|| {
            let delim = if !args.is_empty() {
                args[0].to_string()
            } else {
                " ".to_string()
            };
            let mut parts: Vec<Value> = s
                .rsplit(&delim)
                .map(|p| Value::String(p.to_string()))
                .collect();
            parts.reverse();
            Ok(Value::List(Arc::new(RwLock::new(parts))))
        })()),
        "codepoints" => Some((|| {
            args.require(0, "codepoints")?;
            let points: Vec<Value> = s.chars().map(|c| Value::Int(c as i64)).collect();
            Ok(Value::List(Arc::new(RwLock::new(points))))
        })()),
        "elems" => Some((|| {
            args.require(0, "elems")?;
            let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
            Ok(Value::List(Arc::new(RwLock::new(chars))))
        })()),
        "isalnum" => Some((|| {
            args.require(0, "isalnum")?;
            Ok(Value::Bool(
                !s.is_empty() && s.chars().all(|c| c.is_alphanumeric()),
            ))
        })()),
        "isalpha" => Some((|| {
            args.require(0, "isalpha")?;
            Ok(Value::Bool(
                !s.is_empty() && s.chars().all(|c| c.is_alphabetic()),
            ))
        })()),
        "isdigit" => Some((|| {
            args.require(0, "isdigit")?;
            Ok(Value::Bool(
                !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()),
            ))
        })()),
        "islower" => Some((|| {
            args.require(0, "islower")?;
            Ok(Value::Bool(
                !s.is_empty() && s.chars().any(|c| c.is_alphabetic()) && s == &s.to_lowercase(),
            ))
        })()),
        "isupper" => Some((|| {
            args.require(0, "isupper")?;
            Ok(Value::Bool(
                !s.is_empty() && s.chars().any(|c| c.is_alphabetic()) && s == &s.to_uppercase(),
            ))
        })()),
        "isspace" => Some((|| {
            args.require(0, "isspace")?;
            Ok(Value::Bool(
                !s.is_empty() && s.chars().all(|c| c.is_whitespace()),
            ))
        })()),
        "istitle" => Some((|| {
            args.require(0, "istitle")?;
            if s.is_empty() {
                return Ok(Value::Bool(false));
            }
            let mut cased = false;
            let mut expected_upper = true;
            for c in s.chars() {
                if c.is_uppercase() {
                    if !expected_upper {
                        return Ok(Value::Bool(false));
                    }
                    expected_upper = false;
                    cased = true;
                } else if c.is_lowercase() {
                    if expected_upper {
                        return Ok(Value::Bool(false));
                    }
                    cased = true;
                } else {
                    expected_upper = true;
                }
            }
            Ok(Value::Bool(cased))
        })()),
        _ => None,
    }
}

pub fn call_bound_method(receiver: &Value, method: &str, args: &[Value]) -> Result<Value, String> {
    let result = match receiver {
        Value::List(l) => handle_list_methods(l, method, args),
        Value::Dictionary(d) => handle_dict_methods(d, method, args),
        Value::Set(s) => handle_set_methods(s, method, args),
        Value::String(s) => handle_string_methods(s, method, args),
        _ => None,
    };

    match result {
        Some(res) => res,
        None => {
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
