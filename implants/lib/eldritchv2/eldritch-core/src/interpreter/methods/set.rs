use crate::ast::Value;
use crate::interpreter::introspection::get_type_name;
use super::ArgCheck;
use alloc::collections::BTreeSet;
use alloc::sync::Arc;
use alloc::string::{String, ToString};
use alloc::format;
use spin::RwLock;

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

pub fn handle_set_methods(
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
