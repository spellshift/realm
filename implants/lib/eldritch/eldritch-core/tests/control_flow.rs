mod assert;

#[test]
fn test_if_else() {
    assert::pass(
        r#"
        x = 10
        res = 0
        if x > 5:
            res = 1
        else:
            res = 2
        assert_eq(res, 1)

        if x < 5:
            res = 3
        else:
            res = 4
        assert_eq(res, 4)
    "#,
    );

    assert::pass(
        r#"
        def check(x):
            if x == 0: return "zero"
            elif x == 1: return "one"
            elif x == 2: return "two"
            else: return "many"

        assert_eq(check(0), "zero")
        assert_eq(check(1), "one")
        assert_eq(check(2), "two")
        assert_eq(check(100), "many")
    "#,
    );
}

#[test]
fn test_loops() {
    // Basic iteration
    assert::pass(
        r#"
        sum = 0
        for i in [1, 2, 3, 4]:
            sum = sum + i
        assert_eq(sum, 10)
    "#,
    );

    // Range iteration
    assert::pass(
        r#"
        sum = 0
        for i in range(5):
            sum = sum + i
        assert_eq(sum, 10)
    "#,
    );

    // Break
    assert::pass(
        r#"
        res = 0
        for i in range(10):
            if i == 5:
                break
            res = i
        assert_eq(res, 4)
    "#,
    );

    // Continue
    assert::pass(
        r#"
        count = 0
        for i in range(5):
            if i == 2:
                continue
            count = count + 1
        assert_eq(count, 4)
    "#,
    );
}

#[test]
fn test_loop_scoping_rust() {
    use eldritch_core::{Interpreter, Value};

    // Test 1: Leakage of loop variable
    {
        let mut interp = Interpreter::new();
        let code = r#"
for i in range(3):
    pass
"#;
        let _ = interp.interpret(code).unwrap();
        // i should not exist
        assert!(interp.interpret("i").is_err(), "Loop variable 'i' leaked");
    }

    // Test 2: Leakage of inner variable
    {
        let mut interp = Interpreter::new();
        let code = r#"
for i in range(1):
    x = 100
"#;
        let _ = interp.interpret(code).unwrap();
        // x should not exist
        assert!(interp.interpret("x").is_err(), "Inner variable 'x' leaked");
    }

    // Test 3: Shadowing
    {
        let mut interp = Interpreter::new();
        let code = r#"
i = 999
for i in range(3):
    pass
"#;
        let _ = interp.interpret(code).unwrap();

        let result = interp.interpret("i");
        match result {
            Ok(Value::Int(val)) => assert_eq!(val, 999, "Outer 'i' should be preserved"),
            Ok(v) => panic!("Expected Int(999), got {v:?}"),
            Err(e) => panic!("Outer 'i' should exist: {e}"),
        }
    }
}

#[test]
fn test_recursion_limit() {
    assert::fail(
        r#"
        def crash():
            crash()
        crash()
    "#,
        "Recursion limit exceeded",
    );
}
