use anyhow::Result;
use starlark::values::Value;
pub fn to_json(json: Value) -> Result<String> {
    json.to_json()
}

#[cfg(test)]
mod tests {
    use super::super::super::insert_dict_kv;
    use anyhow::Result;
    use starlark::const_frozen_string;
    use starlark::{
        collections::SmallMap,
        values::{dict::Dict, Heap, Value},
    };

    #[test]
    fn to_json_object() -> Result<()> {
        let test_heap = Heap::new();
        let res = SmallMap::new();
        let mut dict_res = Dict::new(res);
        insert_dict_kv!(dict_res, test_heap, "test", "test".to_string(), String);
        let res = super::to_json(test_heap.alloc(dict_res))?;
        assert_eq!(res, r#"{"test":"test"}"#);
        Ok(())
    }

    #[test]
    fn to_json_list() -> Result<()> {
        let test_heap = Heap::new();
        let vec_val: Vec<Value> = vec![
            test_heap.alloc(1),
            test_heap.alloc("foo"),
            test_heap.alloc(false),
            Value::new_none(),
        ];
        let res = test_heap.alloc(vec_val);
        let res = super::to_json(res)?;
        assert_eq!(res, r#"[1,"foo",false,null]"#);
        Ok(())
    }
}
