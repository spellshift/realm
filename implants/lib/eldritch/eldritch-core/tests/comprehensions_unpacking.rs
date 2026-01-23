#[cfg(test)]
mod tests {
    use eldritch_core::{Interpreter, Value};

    fn eval(code: &str) -> Value {
        let mut interpreter = Interpreter::new();
        interpreter.interpret(code).unwrap()
    }

    fn eval_err(code: &str) -> String {
        let mut interpreter = Interpreter::new();
        match interpreter.interpret(code) {
            Ok(_) => panic!("Expected error"),
            Err(e) => e,
        }
    }

    #[test]
    fn test_dict_comp_unpacking() {
        let code = r#"
items = {"a": 1, "b": 2}
# In eldritch, iterating over dict yields keys.
# To test unpacking, we need a list of tuples.
pairs = [("a", 1), ("b", 2)]
d = {k: v for k, v in pairs}
d
"#;
        let res = eval(code);
        if let Value::Dictionary(d) = res {
            let d = d.read();
            assert_eq!(d.len(), 2);
        } else {
            panic!("Expected Dictionary");
        }
    }

    #[test]
    fn test_dict_comp_unpacking_values() {
        let code = r#"
pairs = [("a", 1), ("b", 2)]
d = {k: v * 2 for k, v in pairs}
assert(d["a"] == 2)
assert(d["b"] == 4)
True
"#;
        let res = eval(code);
        assert_eq!(res, Value::Bool(true));
    }

    #[test]
    fn test_list_comp_unpacking() {
        let code = r#"
pairs = [("a", 1), ("b", 2)]
l = [v for k, v in pairs]
assert(l[0] == 1)
assert(l[1] == 2)
True
"#;
        let res = eval(code);
        assert_eq!(res, Value::Bool(true));
    }

    #[test]
    fn test_set_comp_unpacking() {
        let code = r#"
pairs = [("a", 1), ("b", 2), ("c", 1)]
s = {v for k, v in pairs}
assert(len(s) == 2)
True
"#;
        let res = eval(code);
        assert_eq!(res, Value::Bool(true));
    }

    #[test]
    fn test_unpacking_error_mismatch() {
        let code = r#"
pairs = [("a", 1, 3), ("b", 2)]
d = {k: v for k, v in pairs}
"#;
        let err = eval_err(code);
        assert!(err.contains("Too many (or not enough) values to unpack"));
    }

    #[test]
    fn test_unpacking_error_not_iterable() {
        let code = r#"
pairs = [1, 2]
d = {k: v for k, v in pairs}
"#;
        let err = eval_err(code);
        assert!(err.contains("Cannot unpack non-iterable object"));
    }

    #[test]
    fn test_unpacking_nested() {
        // k, v, x in [(1, 2, 3)] works.
        let code = r#"
triples = [(1, 2, 3)]
l = [x for a, b, x in triples]
assert(l[0] == 3)
True
"#;
        let res = eval(code);
        assert_eq!(res, Value::Bool(true));
    }

    #[test]
    fn test_regular_variable() {
        let code = r#"
l = [1, 2, 3]
l2 = [x*2 for x in l]
assert(l2[0] == 2)
assert(l2[2] == 6)
True
"#;
        let res = eval(code);
        assert_eq!(res, Value::Bool(true));
    }
}
