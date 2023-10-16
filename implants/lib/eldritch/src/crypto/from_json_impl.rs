use anyhow::{anyhow, Result};
use starlark::values::{Heap, Value};

pub fn from_json(starlark_heap: &Heap, json: String) -> Result<Value> {
    let json_data: serde_json::Value = serde_json::from_str(&json).map_err(|e| anyhow!("Error parsing json: {:?}", e))?;
    
    Ok(starlark_heap.alloc(json_data.clone()))
}
