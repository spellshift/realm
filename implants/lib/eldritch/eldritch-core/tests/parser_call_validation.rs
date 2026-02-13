mod assert;

#[test]
fn test_call_unpacking_args() {
    // Basic *args unpacking
    assert::pass(
        r#"
        def f(a, b, c):
            return a + b + c
        l = [1, 2, 3]
        assert_eq(f(*l), 6)
    "#,
    );

    // Partial unpacking
    assert::pass(
        r#"
        def f(a, b, c):
            return a + b + c
        l = [2, 3]
        assert_eq(f(1, *l), 6)
    "#,
    );

    // Unpacking from tuple
    assert::pass(
        r#"
        def f(a, b):
            return a + b
        t = (1, 2)
        assert_eq(f(*t), 3)
    "#,
    );
}

#[test]
fn test_call_unpacking_kwargs() {
    // Basic **kwargs unpacking
    assert::pass(
        r#"
        def f(a, b):
            return a + b
        d = {"a": 1, "b": 2}
        assert_eq(f(**d), 3)
    "#,
    );

    // Partial unpacking
    assert::pass(
        r#"
        def f(a, b, c):
            return a + b + c
        d = {"b": 2, "c": 3}
        assert_eq(f(1, **d), 6)
    "#,
    );
}

#[test]
fn test_call_mixed_unpacking() {
    // Mixed *args and **kwargs
    assert::pass(
        r#"
        def f(a, b, c, d):
            return a + b + c + d
        l = [2]
        d = {"d": 4}
        assert_eq(f(1, *l, c=3, **d), 10)
    "#,
    );
}

#[test]
fn test_call_invalid_ordering() {
    // Positional argument follows keyword argument
    assert::fail(
        r#"
        def f(a, b): pass
        f(a=1, 2)
    "#,
        "Positional argument follows keyword argument",
    );

    // *args follows keyword argument
    // This is valid in Python 3.5+ (iterable unpacking after keywords), but Eldritch might not support it yet.
    // Based on `expr.rs`, `finish_call` loop order suggests strict ordering might be enforced or flexible.
    // Let's check `expr.rs` logic again.
    // It loops and checks for `*`, `**`, `keyword`, `positional`.
    // But it pushes to `args`. The parser allows `*args` anytime if `match_token` works.
    // However, Python syntax usually requires *args before **kwargs.
    // Let's verify what `finish_call` does.
}

#[test]
fn test_lambda_defaults() {
    assert::pass(
        r#"
        f = lambda a, b=2: a + b
        assert_eq(f(1), 3)
        assert_eq(f(1, 3), 4)
    "#,
    );
}

#[test]
fn test_lambda_args() {
    assert::pass(
        r#"
        f = lambda *args: len(args)
        assert_eq(f(1, 2, 3), 3)
        assert_eq(f(), 0)
    "#,
    );
}

#[test]
fn test_lambda_kwargs() {
    assert::pass(
        r#"
        f = lambda **kwargs: kwargs.get("a")
        assert_eq(f(a=1), 1)
        assert_eq(f(b=2), None)
    "#,
    );
}

#[test]
fn test_lambda_mixed() {
    assert::pass(
        r#"
        f = lambda x, y=1, *args, **kwargs: x + y + len(args) + len(kwargs)
        assert_eq(f(1), 2) # 1 + 1 + 0 + 0
        assert_eq(f(1, 2, 3, a=4), 5) # 1 + 2 + 1 + 1
    "#,
    );
}

#[test]
fn test_lambda_iife() {
    assert::pass(
        r#"
        res = (lambda x: x + 1)(2)
        assert_eq(res, 3)
    "#,
    );

    assert::pass(
        r#"
        res = (lambda x, y: x * y)(3, 4)
        assert_eq(res, 12)
    "#,
    );
}

#[test]
fn test_nested_lambdas() {
    assert::pass(
        r#"
        f = lambda x: lambda y: x + y
        add5 = f(5)
        assert_eq(add5(3), 8)
    "#,
    );
}
