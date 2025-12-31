mod assert;

#[test]
fn test_eval_scope_persistence() {
    assert::pass(
        r#"
        x = 10
        eval("x = 20")
        assert_eq(x, 20)

        eval("y = 30")
        assert_eq(y, 30)
    "#,
    );
}

#[test]
fn test_eval_nested_scope() {
    assert::pass(
        r#"
        def test():
            x = 10
            eval("x = 20")
            assert_eq(x, 20)

            # Define new local
            eval("z = 40")
            assert_eq(z, 40)
        test()
    "#,
    );
}

#[test]
fn test_eval_closure_modification() {
    assert::pass(
        r#"
        def outer():
            x = 10
            def inner():
                # eval should be able to see x
                return eval("x")
            return inner()

        assert_eq(outer(), 10)
    "#,
    );
}

#[test]
fn test_eval_return_value() {
    assert::pass(
        r#"
        # Expression returns value
        res = eval("1 + 2")
        assert_eq(res, 3)

        # Statement returns None (or equivalent)
        res = eval("x = 1")
        assert_eq(res, None)
    "#,
    );
}

#[test]
fn test_eval_syntax_error() {
    assert::fail(
        r#"
        eval("1 +")
    "#,
        "SyntaxError",
    );
}

#[test]
fn test_eval_runtime_error() {
    // The error is wrapped in RuntimeError by eval
    assert::fail(
        r#"
        eval("1 / 0")
    "#,
        "divide by zero",
    );
}

#[test]
fn test_eval_argument_types() {
    assert::fail("eval(1)", "argument must be a string");
    assert::fail("eval()", "takes exactly 1 argument");
    assert::fail("eval('x', 'y')", "takes exactly 1 argument");
}

#[test]
fn test_eval_recursion() {
    // Already covered in builtins.rs but let's test a slightly different case
    // Indirect recursion via eval
    assert::fail(
        r#"
        def f():
            eval("f()")
        f()
    "#,
        "Recursion limit exceeded",
    );
}
