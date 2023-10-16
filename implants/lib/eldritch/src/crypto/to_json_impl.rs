use anyhow::Result;
use starlark::values::Value;

pub fn to_json(json: Value) -> Result<String> {
    json.to_json()
}

#[cfg(test)]
mod tests {
    use starlark::{values::{dict::Dict, Heap, Value}, const_frozen_string, collections::SmallMap};
    use anyhow::Result;

    #[test]
    fn to_json_object() -> Result<()> {
        let test_heap = Heap::new();
        let res = SmallMap::new();
        let mut dict_res = Dict::new(res);
        dict_res.insert_hashed(
            const_frozen_string!("test").to_value().get_hashed()?,
            test_heap.alloc_str("test").to_value(),
        );
        let res = super::to_json(test_heap.alloc(dict_res))?;
        assert_eq!(res, r#"{"test":"test"}"#);
        Ok(())
    }

    #[test]
    fn to_json_list() -> Result<()> {
        let test_heap = Heap::new();
        let mut vec_val: Vec<Value> = Vec::new();
        vec_val.push(test_heap.alloc(1));
        vec_val.push(test_heap.alloc("foo"));
        vec_val.push(test_heap.alloc(false));
        vec_val.push(Value::new_none());
        let res = test_heap.alloc(vec_val);
        let res = super::to_json(res)?;
        assert_eq!(res, r#"[1,"foo",false,null]"#);
        Ok(())
    }
}