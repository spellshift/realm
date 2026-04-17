use super::ArgCheck;
use crate::ast::Value;
use crate::interpreter::error::NativeError;
use crate::interpreter::introspection::get_type_name;
use crate::interpreter::operations::{compare_values, values_equal};
use alloc::format;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cmp::Ordering;
use spin::RwLock;

pub fn handle_list_methods(
    l: &Arc<RwLock<Vec<Value>>>,
    method: &str,
    args: &[Value],
) -> Option<Result<Value, NativeError>> {
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
                    return Err(NativeError::type_error(format!(
                        "extend() expects an iterable, got {}",
                        get_type_name(iterable)
                    )));
                }
            }
            Ok(Value::None)
        })()),
        "insert" => Some((|| {
            args.require(2, "insert")?;
            let idx = match args[0] {
                Value::Int(i) => i,
                _ => return Err(NativeError::type_error("insert() index must be an integer")),
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
                Err(NativeError::value_error("list.remove(x): x not in list"))
            }
        })()),
        "index" => Some((|| {
            args.require(1, "index")?;
            let target = &args[0];
            let vec = l.read();
            if let Some(pos) = vec.iter().position(|x| values_equal(x, target)) {
                Ok(Value::Int(pos as i64))
            } else {
                Err(NativeError::value_error("list.index(x): x not in list"))
            }
        })()),
        "pop" => Some((|| {
            args.require(0, "pop")?;
            if let Some(v) = l.write().pop() {
                Ok(v)
            } else {
                Err(NativeError::index_error("pop from empty list"))
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
