use eldritch_core::Interpreter;

#[test]
fn test_forward_reference() {
    let mut interp = Interpreter::new();
    let res = interp.interpret(
        r#"
def b():
    a()

def a():
    print("A")

b()
"#,
    );
    if let Err(e) = res {
        panic!("Failed with: {}", e);
    }
}

#[test]
fn test_redefine() {
    let mut interp = Interpreter::new();
    let res = interp.interpret(
        r#"
def b():
    return a()

def a():
    return "A"

res1 = b()

def a():
    return "A2"

res2 = b()
"#,
    );
    if let Err(e) = res {
        panic!("Failed with: {}", e);
    }
    // Verify values
    let env = interp.env.read();
    let res1 = env.values.get("res1").unwrap();
    let res2 = env.values.get("res2").unwrap();
    assert_eq!(res1, &eldritch_core::Value::String("A".to_string()));
    assert_eq!(res2, &eldritch_core::Value::String("A2".to_string()));
}
