#[cfg(test)]
mod tests {
    use eldritch_core::{Interpreter, Value};

    #[test]
    fn test_raw_strings() {
        let mut interp = Interpreter::new();
        let code = r#"
s1 = r"hello\nworld"
s2 = r"hello\"world"
s3 = r"hello\\world"
s4 = r"path\to\file"
"#;
        if let Err(e) = interp.interpret(code) {
            panic!("Failed to interpret: {:?}", e);
        }

        // Helper to check value
        let mut check = |name: &str, expected: &str| {
            // We interpret `name` to get its value
            match interp.interpret(name) {
                Ok(Value::String(s)) => assert_eq!(s, expected, "Variable {} mismatch", name),
                Ok(v) => panic!("Variable {} is not a string: {:?}", name, v),
                Err(e) => panic!("Failed to lookup {}: {:?}", name, e),
            }
        };

        check("s1", "hello\\nworld");
        check("s2", "hello\\\"world");
        check("s3", "hello\\\\world");
        check("s4", "path\\to\\file");
    }
}
