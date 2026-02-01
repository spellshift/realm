#[cfg(test)]
mod tests {
    use eldritch_core::{Interpreter, Value};

    #[test]
    fn test_multiline_list_basic() {
        let mut interp = Interpreter::new();
        // This fails if newlines are not handled correctly inside lists
        let code = r#"
my_list = ["a",
                 "b",
                  "c",
                ]
"#;
        if let Err(e) = interp.interpret(code) {
            panic!("Failed to interpret: {:?}", e);
        }

        let result = interp.interpret("my_list").unwrap();
        match result {
            Value::List(l) => {
                let l = l.read();
                assert_eq!(l.len(), 3);
                assert_eq!(l[0], Value::String("a".to_string()));
                assert_eq!(l[1], Value::String("b".to_string()));
                assert_eq!(l[2], Value::String("c".to_string()));
            }
            _ => panic!("Expected list, got {:?}", result),
        }
    }

    #[test]
    fn test_multiline_list_variations() {
        let mut interp = Interpreter::new();
        let code = r#"
l1 = [
    1,
    2
]
l2 = [
    3, 4,
    5
]
l3 = [6,
7,
8]
"#;
        if let Err(e) = interp.interpret(code) {
            panic!("Failed to interpret: {:?}", e);
        }

        let mut check_len = |name: &str, len: usize| {
            match interp.interpret(name).unwrap() {
                Value::List(l) => {
                     let l = l.read();
                     assert_eq!(l.len(), len, "List {} has wrong length", name);
                },
                v => panic!("Expected list for {}, got {:?}", name, v),
            }
        };

        check_len("l1", 2);
        check_len("l2", 3);
        check_len("l3", 3);
    }

    #[test]
    fn test_multiline_other_collections() {
        let mut interp = Interpreter::new();
        let code = r#"
my_tuple = (
    1,
    2,
)
my_set = {
    "a",
    "b"
}
my_dict = {
    "key": "value",
    "k2":
    "v2"
}
"#;
        if let Err(e) = interp.interpret(code) {
            panic!("Failed to interpret: {:?}", e);
        }
    }

    #[test]
    fn test_multiline_comments() {
        let mut interp = Interpreter::new();
        let code = r#"
l = [
    # comment 1
    1, # comment 2
    2
    # comment 3
]
"#;
        if let Err(e) = interp.interpret(code) {
            panic!("Failed to interpret: {:?}", e);
        }

        let l = interp.interpret("l").unwrap();
        if let Value::List(l) = l {
            assert_eq!(l.read().len(), 2);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_multiline_empty() {
        let mut interp = Interpreter::new();
        let code = r#"
l = [
]
d = {
}
t = (
)
"#;
        if let Err(e) = interp.interpret(code) {
            panic!("Failed to interpret: {:?}", e);
        }

         let l = interp.interpret("l").unwrap();
        if let Value::List(l) = l {
            assert_eq!(l.read().len(), 0);
        } else {
            panic!("Expected list");
        }
    }
}
