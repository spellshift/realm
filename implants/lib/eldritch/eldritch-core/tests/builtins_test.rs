#[cfg(test)]
mod tests {
    use eldritch_core::Interpreter;
    use eldritch_core::Value;

    #[test]
    fn test_float_arithmetic() {
        let mut interp = Interpreter::new();
        // Remove leading indentation to please parser
        let code = r#"
a = 1.5
b = 2.5
sum = a + b
prod = a * 2
div = b / 2
fdiv = b // 2
"#;
        interp.interpret(code).unwrap();

        let sum = interp.interpret("sum").unwrap();
        if let Value::Float(f) = sum {
            assert_eq!(f, 4.0);
        } else {
            panic!("Expected float, got {sum:?}");
        }

        let prod = interp.interpret("prod").unwrap();
        if let Value::Float(f) = prod {
            assert_eq!(f, 3.0);
        } else {
            panic!("Expected float, got {prod:?}");
        }

        let fdiv = interp.interpret("fdiv").unwrap();
        if let Value::Float(f) = fdiv {
            assert_eq!(f, 1.0);
        } else {
            panic!("Expected float for floor div of float, got {fdiv:?}");
        }
    }

    #[test]
    fn test_abs() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.interpret("abs(-5)").unwrap(), Value::Int(5));

        let val = interp.interpret("abs(-5.5)").unwrap();
        if let Value::Float(f) = val {
            assert_eq!(f, 5.5);
        } else {
            panic!("Expected float, got {val:?}");
        }
    }

    #[test]
    fn test_any_all() {
        let mut interp = Interpreter::new();
        // Any
        assert_eq!(
            interp.interpret("any([0, False, None])").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            interp.interpret("any([0, 1, None])").unwrap(),
            Value::Bool(true)
        );
        assert_eq!(interp.interpret("any([])").unwrap(), Value::Bool(false));

        // All
        assert_eq!(
            interp.interpret("all([1, True, 's'])").unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            interp.interpret("all([1, False, 's'])").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(interp.interpret("all([])").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_dict() {
        let mut interp = Interpreter::new();
        interp.interpret("d = dict(a=1, b=2)").unwrap();
        let val = interp.interpret("d['a']").unwrap();
        assert_eq!(val, Value::Int(1));

        interp.interpret("d2 = dict([('x', 10)], y=20)").unwrap();
        assert_eq!(interp.interpret("d2['x']").unwrap(), Value::Int(10));
        assert_eq!(interp.interpret("d2['y']").unwrap(), Value::Int(20));
    }

    #[test]
    fn test_float_builtin() {
        let mut interp = Interpreter::new();
        match interp.interpret("float('1.5')").unwrap() {
            Value::Float(f) => assert_eq!(f, 1.5),
            v => panic!("Expected float, got {v:?}"),
        }
        match interp.interpret("float(1)").unwrap() {
            Value::Float(f) => assert_eq!(f, 1.0),
            v => panic!("Expected float, got {v:?}"),
        }
    }

    #[test]
    fn test_list_builtin() {
        let mut interp = Interpreter::new();
        let val = interp.interpret("list((1, 2))").unwrap(); // Tuple to list
        if let Value::List(l) = val {
            assert_eq!(l.read().len(), 2);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_min_max() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.interpret("min([1, 2, 3])").unwrap(), Value::Int(1));
        assert_eq!(interp.interpret("max([1, 2, 3])").unwrap(), Value::Int(3));
    }

    #[test]
    fn test_set() {
        let mut interp = Interpreter::new();
        interp.interpret("s = set([1, 2, 2, 3])").unwrap();
        assert_eq!(interp.interpret("len(s)").unwrap(), Value::Int(3));
        assert_eq!(interp.interpret("3 in s").unwrap(), Value::Bool(true));
        assert_eq!(interp.interpret("4 in s").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_sorted() {
        let mut interp = Interpreter::new();
        let val = interp.interpret("sorted([3, 1, 2])").unwrap();
        if let Value::List(l) = val {
            let list = l.read();
            assert_eq!(list[0], Value::Int(1));
            assert_eq!(list[1], Value::Int(2));
            assert_eq!(list[2], Value::Int(3));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_tuple_builtin() {
        let mut interp = Interpreter::new();
        let val = interp.interpret("tuple([1, 2])").unwrap();
        match val {
            Value::Tuple(t) => assert_eq!(t.len(), 2),
            _ => panic!("Expected tuple"),
        }
    }

    #[test]
    fn test_zip() {
        let mut interp = Interpreter::new();
        let val = interp.interpret("zip([1, 2], [3, 4])").unwrap();
        if let Value::List(l) = val {
            let list = l.read();
            assert_eq!(list.len(), 2);
            if let Value::Tuple(t) = &list[0] {
                assert_eq!(t[0], Value::Int(1));
                assert_eq!(t[1], Value::Int(3));
            } else {
                panic!("Expected tuple in zip result");
            }
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_reversed() {
        let mut interp = Interpreter::new();
        let val = interp.interpret("reversed([1, 2, 3])").unwrap();
        if let Value::List(l) = val {
            let list = l.read();
            assert_eq!(list[0], Value::Int(3));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_repr() {
        let mut interp = Interpreter::new();
        match interp.interpret("repr('foo')").unwrap() {
            Value::String(s) => assert_eq!(s, "\"foo\""),
            v => panic!("Expected string, got {v:?}"),
        }
    }
}
