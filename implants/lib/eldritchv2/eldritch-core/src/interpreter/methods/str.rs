use super::ArgCheck;
use crate::ast::Value;
use crate::interpreter::introspection::is_truthy;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use spin::RwLock;

pub fn handle_string_methods(
    s: &str,
    method: &str,
    args: &[Value],
) -> Option<Result<Value, String>> {
    match method {
        "split" => Some({
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
        }),
        "splitlines" => Some({
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
        }),
        "strip" => Some((|| {
            args.require_range(0, 1, "strip")?;
            if args.is_empty() {
                Ok(Value::String(s.trim().to_string()))
            } else {
                let chars_val = args[0].to_string();
                let chars: Vec<char> = chars_val.chars().collect();
                Ok(Value::String(s.trim_matches(&chars[..]).to_string()))
            }
        })()),
        "lstrip" => Some((|| {
            args.require_range(0, 1, "lstrip")?;
            if args.is_empty() {
                Ok(Value::String(s.trim_start().to_string()))
            } else {
                let chars_val = args[0].to_string();
                let chars: Vec<char> = chars_val.chars().collect();
                Ok(Value::String(s.trim_start_matches(&chars[..]).to_string()))
            }
        })()),
        "rstrip" => Some((|| {
            args.require_range(0, 1, "rstrip")?;
            if args.is_empty() {
                Ok(Value::String(s.trim_end().to_string()))
            } else {
                let chars_val = args[0].to_string();
                let chars: Vec<char> = chars_val.chars().collect();
                Ok(Value::String(s.trim_end_matches(&chars[..]).to_string()))
            }
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
        "rsplit" => Some({
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
        }),
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
                !s.is_empty() && s.chars().any(|c| c.is_alphabetic()) && s == s.to_lowercase(),
            ))
        })()),
        "isupper" => Some((|| {
            args.require(0, "isupper")?;
            Ok(Value::Bool(
                !s.is_empty() && s.chars().any(|c| c.is_alphabetic()) && s == s.to_uppercase(),
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
