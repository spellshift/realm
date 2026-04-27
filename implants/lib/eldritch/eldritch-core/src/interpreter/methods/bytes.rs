use super::ArgCheck;
use crate::ast::Value;
use crate::interpreter::introspection::is_truthy;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

pub fn handle_bytes_methods(
    b: &[u8],
    method: &str,
    args: &[Value],
) -> Option<Result<Value, String>> {
    match method {
        "decode" => Some((|| {
            args.require(0, "decode")?;
            match String::from_utf8(b.to_vec()) {
                Ok(s) => Ok(Value::String(s)),
                Err(e) => Err(alloc::format!("UnicodeDecodeError: {}", e)),
            }
        })()),
        "split" => Some((|| {
            args.require_range(0, 1, "split")?;
            let parts: Vec<Value> = if args.is_empty() {
                b.split(|&c| (c as char).is_ascii_whitespace())
                    .filter(|p| !p.is_empty())
                    .map(|p| Value::Bytes(p.to_vec()))
                    .collect()
            } else {
                match &args[0] {
                    Value::Bytes(delim) => {
                        if delim.is_empty() {
                            return Err("ValueError: empty separator".into());
                        }
                        let mut result = Vec::new();
                        let mut last = 0;
                        let mut i = 0;
                        while i + delim.len() <= b.len() {
                            if &b[i..i + delim.len()] == delim {
                                result.push(Value::Bytes(b[last..i].to_vec()));
                                i += delim.len();
                                last = i;
                            } else {
                                i += 1;
                            }
                        }
                        result.push(Value::Bytes(b[last..].to_vec()));
                        result
                    }
                    _ => return Err("TypeError: expected bytes".into()),
                }
            };
            Ok(Value::List(Arc::new(RwLock::new(parts))))
        })()),
        "splitlines" => Some((|| {
            args.require_range(0, 1, "splitlines")?;
            let keepends = if !args.is_empty() {
                is_truthy(&args[0])
            } else {
                false
            };

            let mut lines = Vec::new();
            let mut start = 0;
            let mut i = 0;
            while i < b.len() {
                if b[i] == b'\n' || b[i] == b'\r' {
                    let mut end = i + 1;
                    if b[i] == b'\r' && i + 1 < b.len() && b[i + 1] == b'\n' {
                        end += 1;
                    }
                    if keepends {
                        lines.push(Value::Bytes(b[start..end].to_vec()));
                    } else {
                        lines.push(Value::Bytes(b[start..i].to_vec()));
                    }
                    start = end;
                    i = end;
                } else {
                    i += 1;
                }
            }
            if start < b.len() {
                lines.push(Value::Bytes(b[start..].to_vec()));
            }

            Ok(Value::List(Arc::new(RwLock::new(lines))))
        })()),
        "rsplit" => Some((|| {
            args.require_range(0, 1, "rsplit")?;
            let parts: Vec<Value> = if args.is_empty() {
                b.split(|&c| (c as char).is_ascii_whitespace())
                    .filter(|p| !p.is_empty())
                    .map(|p| Value::Bytes(p.to_vec()))
                    .collect()
            } else {
                match &args[0] {
                    Value::Bytes(delim) => {
                        if delim.is_empty() {
                            return Err("ValueError: empty separator".into());
                        }
                        let mut result = Vec::new();
                        let mut last = b.len();
                        if b.len() >= delim.len() {
                            let mut i = b.len() - delim.len();
                            loop {
                                if &b[i..i + delim.len()] == delim {
                                    result.push(Value::Bytes(b[i + delim.len()..last].to_vec()));
                                    last = i;
                                    if i < delim.len() {
                                        break;
                                    }
                                    i -= delim.len();
                                } else {
                                    if i == 0 {
                                        break;
                                    }
                                    i -= 1;
                                }
                            }
                        }
                        result.push(Value::Bytes(b[..last].to_vec()));
                        result.reverse();
                        result
                    }
                    _ => return Err("TypeError: expected bytes".into()),
                }
            };
            Ok(Value::List(Arc::new(RwLock::new(parts))))
        })()),
        "strip" => Some((|| {
            args.require_range(0, 1, "strip")?;
            if args.is_empty() {
                let start = b
                    .iter()
                    .position(|&c| !(c as char).is_ascii_whitespace())
                    .unwrap_or(b.len());
                let end = b[start..]
                    .iter()
                    .rposition(|&c| !(c as char).is_ascii_whitespace())
                    .map(|pos| start + pos + 1)
                    .unwrap_or(start);
                Ok(Value::Bytes(b[start..end].to_vec()))
            } else {
                match &args[0] {
                    Value::Bytes(chars) => {
                        let start = b.iter().position(|c| !chars.contains(c)).unwrap_or(b.len());
                        let end = b[start..]
                            .iter()
                            .rposition(|c| !chars.contains(c))
                            .map(|pos| start + pos + 1)
                            .unwrap_or(start);
                        Ok(Value::Bytes(b[start..end].to_vec()))
                    }
                    _ => Err("TypeError: expected bytes".into()),
                }
            }
        })()),
        "lstrip" => Some((|| {
            args.require_range(0, 1, "lstrip")?;
            if args.is_empty() {
                let start = b
                    .iter()
                    .position(|&c| !(c as char).is_ascii_whitespace())
                    .unwrap_or(b.len());
                Ok(Value::Bytes(b[start..].to_vec()))
            } else {
                match &args[0] {
                    Value::Bytes(chars) => {
                        let start = b.iter().position(|c| !chars.contains(c)).unwrap_or(b.len());
                        Ok(Value::Bytes(b[start..].to_vec()))
                    }
                    _ => Err("TypeError: expected bytes".into()),
                }
            }
        })()),
        "rstrip" => Some((|| {
            args.require_range(0, 1, "rstrip")?;
            if args.is_empty() {
                let end = b
                    .iter()
                    .rposition(|&c| !(c as char).is_ascii_whitespace())
                    .map(|pos| pos + 1)
                    .unwrap_or(0);
                Ok(Value::Bytes(b[..end].to_vec()))
            } else {
                match &args[0] {
                    Value::Bytes(chars) => {
                        let end = b
                            .iter()
                            .rposition(|c| !chars.contains(c))
                            .map(|pos| pos + 1)
                            .unwrap_or(0);
                        Ok(Value::Bytes(b[..end].to_vec()))
                    }
                    _ => Err("TypeError: expected bytes".into()),
                }
            }
        })()),
        "startswith" => Some((|| {
            args.require(1, "startswith")?;
            match &args[0] {
                Value::Bytes(prefix) => Ok(Value::Bool(b.starts_with(prefix))),
                _ => Err("TypeError: expected bytes".into()),
            }
        })()),
        "endswith" => Some((|| {
            args.require(1, "endswith")?;
            match &args[0] {
                Value::Bytes(suffix) => Ok(Value::Bool(b.ends_with(suffix))),
                _ => Err("TypeError: expected bytes".into()),
            }
        })()),
        "removeprefix" => Some((|| {
            args.require(1, "removeprefix")?;
            match &args[0] {
                Value::Bytes(prefix) => {
                    if b.starts_with(prefix) {
                        Ok(Value::Bytes(b[prefix.len()..].to_vec()))
                    } else {
                        Ok(Value::Bytes(b.to_vec()))
                    }
                }
                _ => Err("TypeError: expected bytes".into()),
            }
        })()),
        "removesuffix" => Some((|| {
            args.require(1, "removesuffix")?;
            match &args[0] {
                Value::Bytes(suffix) => {
                    if b.ends_with(suffix) {
                        Ok(Value::Bytes(b[..b.len() - suffix.len()].to_vec()))
                    } else {
                        Ok(Value::Bytes(b.to_vec()))
                    }
                }
                _ => Err("TypeError: expected bytes".into()),
            }
        })()),
        "find" => Some((|| {
            args.require(1, "find")?;
            match &args[0] {
                Value::Bytes(sub) => {
                    if sub.is_empty() {
                        return Ok(Value::Int(0));
                    }
                    if sub.len() <= b.len() {
                        for i in 0..=b.len() - sub.len() {
                            if &b[i..i + sub.len()] == sub {
                                return Ok(Value::Int(i as i64));
                            }
                        }
                    }
                    Ok(Value::Int(-1))
                }
                _ => Err("TypeError: expected bytes".into()),
            }
        })()),
        "index" => Some((|| {
            args.require(1, "index")?;
            match &args[0] {
                Value::Bytes(sub) => {
                    if sub.is_empty() {
                        return Ok(Value::Int(0));
                    }
                    if sub.len() <= b.len() {
                        for i in 0..=b.len() - sub.len() {
                            if &b[i..i + sub.len()] == sub {
                                return Ok(Value::Int(i as i64));
                            }
                        }
                    }
                    Err("ValueError: substring not found".into())
                }
                _ => Err("TypeError: expected bytes".into()),
            }
        })()),
        "rfind" => Some((|| {
            args.require(1, "rfind")?;
            match &args[0] {
                Value::Bytes(sub) => {
                    if sub.is_empty() {
                        return Ok(Value::Int(b.len() as i64));
                    }
                    if sub.len() <= b.len() {
                        for i in (0..=b.len() - sub.len()).rev() {
                            if &b[i..i + sub.len()] == sub {
                                return Ok(Value::Int(i as i64));
                            }
                        }
                    }
                    Ok(Value::Int(-1))
                }
                _ => Err("TypeError: expected bytes".into()),
            }
        })()),
        "rindex" => Some((|| {
            args.require(1, "rindex")?;
            match &args[0] {
                Value::Bytes(sub) => {
                    if sub.is_empty() {
                        return Ok(Value::Int(b.len() as i64));
                    }
                    if sub.len() <= b.len() {
                        for i in (0..=b.len() - sub.len()).rev() {
                            if &b[i..i + sub.len()] == sub {
                                return Ok(Value::Int(i as i64));
                            }
                        }
                    }
                    Err("ValueError: substring not found".into())
                }
                _ => Err("TypeError: expected bytes".into()),
            }
        })()),
        "count" => Some((|| {
            args.require(1, "count")?;
            match &args[0] {
                Value::Bytes(sub) => {
                    if sub.is_empty() {
                        return Ok(Value::Int((b.len() + 1) as i64));
                    }
                    let mut count = 0;
                    let mut i = 0;
                    while i + sub.len() <= b.len() {
                        if &b[i..i + sub.len()] == sub {
                            count += 1;
                            i += sub.len();
                        } else {
                            i += 1;
                        }
                    }
                    Ok(Value::Int(count))
                }
                _ => Err("TypeError: expected bytes".into()),
            }
        })()),
        "replace" => Some((|| {
            args.require(2, "replace")?;
            match (&args[0], &args[1]) {
                (Value::Bytes(old), Value::Bytes(new)) => {
                    if old.is_empty() {
                        let mut result = Vec::new();
                        for &byte in b {
                            result.extend_from_slice(new);
                            result.push(byte);
                        }
                        result.extend_from_slice(new);
                        return Ok(Value::Bytes(result));
                    }
                    let mut result = Vec::new();
                    let mut i = 0;
                    while i < b.len() {
                        if i + old.len() <= b.len() && &b[i..i + old.len()] == old {
                            result.extend_from_slice(new);
                            i += old.len();
                        } else {
                            result.push(b[i]);
                            i += 1;
                        }
                    }
                    Ok(Value::Bytes(result))
                }
                _ => Err("TypeError: expected bytes".into()),
            }
        })()),
        "join" => Some((|| {
            args.require(1, "join")?;
            match &args[0] {
                Value::List(l) => {
                    let list = l.read();
                    let mut result = Vec::new();
                    for (i, v) in list.iter().enumerate() {
                        if i > 0 {
                            result.extend_from_slice(b);
                        }
                        match v {
                            Value::Bytes(bb) => result.extend_from_slice(bb),
                            _ => return Err("TypeError: join() expects list of bytes".to_string()),
                        }
                    }
                    Ok(Value::Bytes(result))
                }
                _ => Err("TypeError: join() expects a list".into()),
            }
        })()),
        "partition" => Some((|| {
            args.require(1, "partition")?;
            match &args[0] {
                Value::Bytes(sep) => {
                    if sep.is_empty() {
                        return Err("ValueError: empty separator".into());
                    }
                    if sep.len() <= b.len() {
                        for i in 0..=b.len() - sep.len() {
                            if &b[i..i + sep.len()] == sep {
                                return Ok(Value::Tuple(vec![
                                    Value::Bytes(b[..i].to_vec()),
                                    Value::Bytes(sep.clone()),
                                    Value::Bytes(b[i + sep.len()..].to_vec()),
                                ]));
                            }
                        }
                    }
                    Ok(Value::Tuple(vec![
                        Value::Bytes(b.to_vec()),
                        Value::Bytes(Vec::new()),
                        Value::Bytes(Vec::new()),
                    ]))
                }
                _ => Err("TypeError: expected bytes".into()),
            }
        })()),
        "rpartition" => Some((|| {
            args.require(1, "rpartition")?;
            match &args[0] {
                Value::Bytes(sep) => {
                    if sep.is_empty() {
                        return Err("ValueError: empty separator".into());
                    }
                    if sep.len() <= b.len() {
                        for i in (0..=b.len() - sep.len()).rev() {
                            if &b[i..i + sep.len()] == sep {
                                return Ok(Value::Tuple(vec![
                                    Value::Bytes(b[..i].to_vec()),
                                    Value::Bytes(sep.clone()),
                                    Value::Bytes(b[i + sep.len()..].to_vec()),
                                ]));
                            }
                        }
                    }
                    Ok(Value::Tuple(vec![
                        Value::Bytes(Vec::new()),
                        Value::Bytes(Vec::new()),
                        Value::Bytes(b.to_vec()),
                    ]))
                }
                _ => Err("TypeError: expected bytes".into()),
            }
        })()),
        _ => None,
    }
}
