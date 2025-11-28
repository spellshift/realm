use eldritchv2::{Interpreter, Value};

#[test]
fn test_dict() {
    let code = r#"
mydict = {"a": 1, "b": 2}
return mydict["a"] + mydict["b"]
"#;
    let mut i = Interpreter::new();
    let res = i.interpret(code).unwrap();
    assert_eq!(res, Value::Int(3));
}

#[test]
fn test_fstring() {
    let code = r#"
x = 5
y = "ok"
mystr = f"{x} is {y}"
return mystr
"#;
    let mut i = Interpreter::new();
    let res = i.interpret(code).unwrap();
    assert_eq!(res, Value::String("5 is ok".to_string()));
}
