use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::format;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use spin::RwLock;

/// `zip(*iterables)`: Returns an iterator of tuples.
///
/// Returns a list of tuples, where the i-th tuple contains the i-th element from each of the argument sequences or iterables.
/// The returned list is truncated to the length of the shortest argument sequence.
///
/// **Parameters**
/// - `*iterables` (Iterable): Iterables to zip together.
pub fn builtin_zip(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::List(Arc::new(RwLock::new(Vec::new()))));
    }

    let mut iterators: Vec<Vec<Value>> = Vec::new();
    for arg in args {
        let items = match arg {
            Value::List(l) => l.read().clone(),
            Value::Tuple(t) => t.clone(),
            Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
            Value::Set(s) => s.read().iter().cloned().collect(),
            Value::Dictionary(d) => d
                .read()
                .keys()
                .cloned()
                .collect(),
            _ => return Err(format!("'{}' object is not iterable", get_type_name(arg))),
        };
        iterators.push(items);
    }

    let min_len = iterators.iter().map(|v| v.len()).min().unwrap_or(0);
    let mut result = Vec::with_capacity(min_len);

    for i in 0..min_len {
        let mut tuple_items = Vec::with_capacity(iterators.len());
        for iter in &iterators {
            tuple_items.push(iter[i].clone());
        }
        result.push(Value::Tuple(tuple_items));
    }

    Ok(Value::List(Arc::new(RwLock::new(result))))
}
