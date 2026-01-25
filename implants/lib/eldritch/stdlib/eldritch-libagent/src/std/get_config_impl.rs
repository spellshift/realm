use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use eldritch_core::Value;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;

pub fn get_config(agent: Arc<dyn Agent>) -> Result<BTreeMap<String, Value>, String> {
    let config = agent.get_config()?;
    let mut result = BTreeMap::new();
    for (k, v) in config {
        // Try to parse numbers, otherwise keep as string
        if let Ok(i) = v.parse::<i64>() {
            result.insert(k, Value::Int(i));
        } else if let Ok(b) = v.parse::<bool>() {
            result.insert(k, Value::Bool(b));
        } else {
            result.insert(k, Value::String(v));
        }
    }
    Ok(result)
}
