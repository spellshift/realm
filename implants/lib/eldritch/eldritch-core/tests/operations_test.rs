use eldritch_core::{Interpreter, Value};
#[test]
fn test_slice_indices_adjust() {
    // We can't directly call operations::adjust_slice_indices unless we expose it.
    // It's pub, but inside `interpreter` module which is private?
    // In lib.rs: `mod interpreter;`. `pub use interpreter::{Interpreter, ...}`.
    // But `operations` is `pub mod` inside interpreter.
    // So `eldritch_core::interpreter::operations` should be accessible if `interpreter` is accessible?
    // `mod interpreter` is private in lib.rs.
    // So we can only test via behavior.

    let mut interp = Interpreter::new();

    // List slicing
    let code = "x = [0, 1, 2, 3, 4]; x[1:4]";
    let val = interp.interpret(code).unwrap();
    if let Value::List(l) = val {
        let list = l.read();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0], Value::Int(1));
        assert_eq!(list[2], Value::Int(3));
    } else {
        panic!("Expected list");
    }

    // Negative indices
    let code = "x = [0, 1, 2, 3, 4]; x[-2:]";
    let val = interp.interpret(code).unwrap();
    if let Value::List(l) = val {
        let list = l.read();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], Value::Int(3));
        assert_eq!(list[1], Value::Int(4));
    }

    // Step
    let code = "x = [0, 1, 2, 3, 4]; x[::2]";
    let val = interp.interpret(code).unwrap();
    if let Value::List(l) = val {
        let list = l.read();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0], Value::Int(0));
        assert_eq!(list[1], Value::Int(2));
        assert_eq!(list[2], Value::Int(4));
    }

    // Negative Step
    let code = "x = [0, 1, 2, 3, 4]; x[::-1]";
    let val = interp.interpret(code).unwrap();
    if let Value::List(l) = val {
        let list = l.read();
        assert_eq!(list.len(), 5);
        assert_eq!(list[0], Value::Int(4));
        assert_eq!(list[4], Value::Int(0));
    }
}

#[test]
fn test_concatenation_optimization() {
    // Logic test: behavior should remain same.
    // We can't easily verify memory reuse from integration test without unsafe pointer checks.
    // But we verify correctness.
    let mut interp = Interpreter::new();

    let code = "x = [1, 2]; y = [3, 4]; z = x + y; z";
    let val = interp.interpret(code).unwrap();
    if let Value::List(l) = val {
        let list = l.read();
        assert_eq!(list.len(), 4);
        assert_eq!(list[0], Value::Int(1));
        assert_eq!(list[3], Value::Int(4));
    }

    // Test that 'x' is NOT mutated (if it was reused incorrectly)
    // We can't access variable lookup directly via public API, but we can interpret again.
    let val_x = interp.interpret("x").unwrap();
    if let Value::List(l) = val_x {
        let list = l.read();
        assert_eq!(list.len(), 2);
    }

    // Test rvalue optimization scenario
    // [1, 2] + [3, 4] -> The first list is temporary.
    let code = "[1, 2] + [3, 4]";
    let val = interp.interpret(code).unwrap();
    if let Value::List(l) = val {
        let list = l.read();
        assert_eq!(list.len(), 4);
    }
}

#[test]
fn test_mixed_comparisons() {
    let mut interp = Interpreter::new();

    assert_eq!(interp.interpret("1 == 1.0").unwrap(), Value::Bool(true));
    assert_eq!(interp.interpret("1 < 1.1").unwrap(), Value::Bool(true));
    assert_eq!(interp.interpret("2 > 1.9").unwrap(), Value::Bool(true));

    // Different types
    // "1 < 'a'" -> Error or False? operations.rs says Error.
    let res = interp.interpret("1 < 'a'");
    assert!(res.is_err());
}
