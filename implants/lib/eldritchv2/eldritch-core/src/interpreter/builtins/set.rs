use crate::ast::{Environment, Value};
use crate::interpreter::introspection::get_type_name;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::string::ToString;
use spin::RwLock;

#[allow(clippy::mutable_key_type)]
/// `set([iterable])`: Creates a set.
///
/// If no argument is given, the constructor creates a new empty set.
/// The argument must be an iterable if specified.
pub fn builtin_set(_env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::Set(Arc::new(RwLock::new(BTreeSet::new()))));
    }
    if args.len() != 1 {
        return Err(format!(
            "set() takes at most 1 argument ({} given)",
            args.len()
        ));
    }

    let items = match &args[0] {
        Value::List(l) => l.read().clone(),
        Value::Tuple(t) => t.clone(),
        Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
        Value::Set(s) => s.read().iter().cloned().collect(),
        Value::Dictionary(d) => d
            .read()
            .keys()
            .cloned()
            .collect(),
        _ => {
            return Err(format!(
                "'{}' object is not iterable",
                get_type_name(&args[0])
            ))
        }
    };

    #[allow(clippy::mutable_key_type)]
    let mut set = BTreeSet::new();
    for item in items {
        // Here we rely on Value implementing Ord.
        // Mutable types (List, Dict, Set) are compared by content in my implementation.
        // This deviates from Python (unhashable), but Eldritch V2 handles it this way for now.
        set.insert(item);
    }

    Ok(Value::Set(Arc::new(RwLock::new(set))))
}
