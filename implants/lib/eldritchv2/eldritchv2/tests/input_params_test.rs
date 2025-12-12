#[cfg(test)]
mod tests {
    use eldritch_core::Value;
    use eldritchv2::Interpreter;
    use spin::RwLock;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    #[test]
    fn test_input_params() {
        let mut interp = Interpreter::new();

        // Create input_params dictionary
        #[allow(clippy::mutable_key_type)]
        let mut params = BTreeMap::new();
        params.insert(
            Value::String("key1".to_string()),
            Value::String("value1".to_string()),
        );
        params.insert(Value::String("key2".to_string()), Value::Int(42));

        let params_val = Value::Dictionary(Arc::new(RwLock::new(params)));

        // Define variable in interpreter
        interp.define_variable("input_params", params_val);

        // Verify access within script - ensure no leading indentation
        let code = "val1 = input_params['key1']\nval2 = input_params['key2']";

        interp.interpret(code).unwrap();

        assert_eq!(
            interp.interpret("val1").unwrap(),
            Value::String("value1".to_string())
        );
        assert_eq!(interp.interpret("val2").unwrap(), Value::Int(42));
    }
}
