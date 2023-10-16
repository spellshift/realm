use anyhow::Result;
use starlark::values::Value;

pub fn to_json(json: Value) -> Result<String> {
    Ok(json.to_str())
}
