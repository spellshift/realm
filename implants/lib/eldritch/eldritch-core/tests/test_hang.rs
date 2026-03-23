use eldritch_core::Interpreter;

#[test]
fn test_hang_deadlock() {
    let mut interp = Interpreter::new();
    let code = r#"
def a():
    def b():
        fail("error")
    b()
a()
"#;
    let res = interp.interpret(code);
    assert!(res.is_err());

    // Interpreter should still be in a valid state
    let code2 = r#"
def c():
    print("c")
"#;
    let res2 = interp.interpret(code2);
    assert!(res2.is_ok());
}
