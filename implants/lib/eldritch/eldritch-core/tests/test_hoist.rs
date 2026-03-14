use eldritch_core::Interpreter;

#[test]
fn test_hoist() {
    let mut interp = Interpreter::new();
    let res = interp.interpret(
        "
def b():
    a()

b()

def a():
    print(\"A\")
",
    );
    println!("result is {:?}", res);
    assert!(res.is_ok());
}
