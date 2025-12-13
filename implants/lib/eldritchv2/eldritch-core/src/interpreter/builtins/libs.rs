use crate::ast::{Environment, Value};
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::RwLock;

/// `libs()`: Lists all registered libraries.
///
/// Returns a list of strings representing the names of all libraries loaded
/// in the current environment scope chain.
pub fn builtin_libs(env: &Arc<RwLock<Environment>>, args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("libs() takes no arguments".to_string());
    }

    let mut names = BTreeSet::new();
    let mut current_env = Some(env.clone());

    while let Some(env_arc) = current_env {
        let env_ref = env_arc.read();
        for lib in &env_ref.libraries {
            names.insert(lib.clone());
        }
        current_env = env_ref.parent.clone();
    }

    let val_list: Vec<Value> = names.into_iter().map(Value::String).collect();
    Ok(Value::List(Arc::new(RwLock::new(val_list))))
}
