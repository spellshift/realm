use eldritch_core::Interpreter;

#[test]
fn test_nested() {
    let mut interp = Interpreter::new();
    let res = interp.interpret(
        "
def c():
    def b():
        a()
    def a():
        print(\"A\")
    b()

c()
",
    );
    if let Err(e) = res {
        panic!("Failed with: {}", e);
    }
}
