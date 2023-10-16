use anyhow::{anyhow, Result};
use starlark::values::{Heap, Value};

pub fn from_json(starlark_heap: &Heap, json: String) -> Result<Value> {
    let json_data: serde_json::Value = serde_json::from_str(&json).map_err(|e| anyhow!("Error parsing json: {:?}", e))?;
    
    Ok(starlark_heap.alloc(json_data.clone()))
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use starlark::values::{Heap, Value};

    #[test]
    fn test_from_json_object() -> anyhow::Result<()>{
        let test_heap = Heap::new();
        let res = super::from_json(&test_heap, r#"{"test": "test"}"#.to_string())?;
        let res_value = test_heap.alloc(json!({"test": "test"}));
        assert_eq!(res, res_value);
        Ok(())
    }

    #[test]
    fn test_from_json_list() -> anyhow::Result<()>{
        let test_heap = Heap::new();
        let res = super::from_json(&test_heap, r#"[1, "foo", false, null]"#.to_string())?;
        let res_value = test_heap.alloc(json!([1, "foo", false, null]));
        assert_eq!(res, res_value);
        Ok(())
    }

    #[test]
    fn test_from_json_invalid() -> anyhow::Result<()>{
        let test_heap = Heap::new();
        let res = super::from_json(&test_heap, r#"{"test":"#.to_string());
        assert!(res.is_err());
        Ok(())
    }
}