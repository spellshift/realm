
use eldritch_core::{Interpreter, Value};

#[test]
fn test_bytes_subscript() {
    let mut interp = Interpreter::new();
    let code = r#"
b = b"hello world"
a = b[0]
b_slice = b[0:5]
b_slice_step = b[::2]
"#;

    interp.interpret(code).unwrap();

    // Check results by interpreting expressions that return the values
    let a = interp.interpret("a").unwrap();
    if let Value::Int(i) = a {
        assert_eq!(i, 104); // 'h'
    } else {
        panic!("b[0] should be Int, got {:?}", a);
    }

    let b_slice = interp.interpret("b_slice").unwrap();
    if let Value::Bytes(b) = b_slice {
        assert_eq!(b, b"hello".to_vec());
    } else {
        panic!("b[0:5] should be Bytes, got {:?}", b_slice);
    }

    let b_slice_step = interp.interpret("b_slice_step").unwrap();
    if let Value::Bytes(b) = b_slice_step {
        assert_eq!(b, b"hlowrd".to_vec());
    } else {
        panic!("b[::2] should be Bytes, got {:?}", b_slice_step);
    }
}
