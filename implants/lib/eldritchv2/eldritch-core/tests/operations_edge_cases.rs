use eldritch_core::{Interpreter, Value};
mod assert;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

#[test]
fn test_bitwise_ops_edge_cases() {
    let mut interp = Interpreter::new();

    // Test Set invalid bitwise ops
    // Sets support &, |, ^
    // Should fail for <<, >>
    let code = "s = {1}; s << 2";
    let res = interp.interpret(code);
    assert!(res.is_err());
    let err = res.err().unwrap().to_string();
    // Lowercase names
    assert!(err.contains("unsupported operand type(s) for <<: 'set' and 'int'"));

    // Test Dict invalid bitwise ops
    // Dicts support | (union)
    // Should fail for &, ^, <<, >>
    let code = "d = {'a': 1}; d & {'b': 2}";
    let res = interp.interpret(code);
    assert!(res.is_err());
    let err = res.err().unwrap().to_string();
    assert!(err.contains("unsupported operand type(s) for &: 'dict' and 'dict'"));

    // Test Type Mismatch
    let code = "1 & 'a'";
    let res = interp.interpret(code);
    assert!(res.is_err());
    let err = res.err().unwrap().to_string();
    assert!(err.contains("unsupported operand type(s) for &: 'int' and 'string'"));
}

#[test]
fn test_arithmetic_ops_edge_cases() {
    let mut interp = Interpreter::new();

    // Type mismatch for arithmetic
    let code = "1 + 'a'";
    let res = interp.interpret(code);
    assert!(res.is_err());
    let err = res.err().unwrap().to_string();
    // Lowercase names
    assert!(err.contains("unsupported operand type(s) for +: 'int' and 'string'"));

    let code = "1 - 'a'";
    let res = interp.interpret(code);
    assert!(res.is_err());

     let code = "1 * 'a'"; // Actually this might work if String * Int is supported?
     // 1 * 'a' -> 'a' (Repetition)
     // Let's check string repetition logic in arithmetic.rs
     // Repetition handles String * Int and Int * String.
     let res = interp.interpret(code);
     assert_eq!(res.unwrap(), Value::String(alloc::string::String::from("a")));

     // What about float repetition?
     let code = "1.5 * 'a'";
     let res = interp.interpret(code);
     assert!(res.is_err()); // Float repetition not supported
}

#[test]
fn test_comparison_edge_cases() {
    let mut interp = Interpreter::new();

    // Compare mismatched types that are not numbers
    let code = "'a' < [1]";
    let res = interp.interpret(code);
    assert!(res.is_err());
    let err = res.err().unwrap().to_string();
    // Lowercase names
    assert!(err.contains("'<' not supported between instances of 'string' and 'list'"));
}
