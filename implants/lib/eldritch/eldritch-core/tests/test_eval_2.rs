use eldritch_core::Interpreter;

#[test]
fn test_manual() {
    let mut interp = Interpreter::new();
    let res = interp.interpret(
        "
def b():
    a()

def a():
    print(\"A\")

b()
",
    );
    println!("result is {:?}", res);
    assert!(res.is_ok());
}
