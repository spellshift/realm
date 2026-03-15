use eldritch_core::Interpreter;

#[test]
fn test_manual() {
    let mut interp = Interpreter::new();
    let res = interp.interpret("def b():\n    a()");
    println!("def b: {:?}", res);

    let res = interp.interpret("def a():\n    print(\"A\")");
    println!("def a: {:?}", res);

    let res = interp.interpret("b()");
    println!("result is {:?}", res);
}
