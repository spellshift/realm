mod assert;

#[test]
fn test_functions_basic() {
    assert::pass(
        r#"
        def add(a, b):
            return a + b

        assert_eq(add(10, 5), 15)

        def do_nothing():
            return None

        assert_eq(do_nothing(), None)
    "#,
    );
}

#[test]
fn test_function_arguments() {
    // Default arguments
    assert::pass(
        r#"
        def f(a, b=10):
            return a + b
        assert_eq(f(5), 15)
        assert_eq(f(5, 20), 25)
    "#,
    );

    // Keyword arguments
    assert::pass(
        r#"
        def f(a, b, c=5):
            return a + b + c
        assert_eq(f(1, 2), 8)
        assert_eq(f(a=1, b=2), 8)
        assert_eq(f(b=2, a=1), 8)
        assert_eq(f(1, c=10, b=2), 13)
    "#,
    );

    // *args
    assert::pass(
        r#"
        def f(a, *args):
            return len(args)
        assert_eq(f(1), 0)
        assert_eq(f(1, 2, 3), 2)
    "#,
    );

    // **kwargs
    assert::pass(
        r#"
        def f(a, **kwargs):
            return kwargs["x"]
        assert_eq(f(1, x=10), 10)
    "#,
    );

    // Mixed
    assert::pass(
        r#"
        def f(a, b=2, *args, **kwargs):
            return a + b + len(args) + len(kwargs)
        assert_eq(f(1, 3, 4, x=1), 6)
    "#,
    );
}

#[test]
fn test_closures() {
    assert::pass(
        r#"
        def make_adder(n):
            def adder(x):
                return x + n
            return adder

        add5 = make_adder(5)
        assert_eq(add5(3), 8)
    "#,
    );

    assert::pass(
        r#"
        x = 0
        def inc():
            x = x + 1
            return x
        assert_eq(inc(), 1)
        assert_eq(inc(), 2)
    "#,
    );
}

#[test]
fn test_lambdas() {
    assert::pass(
        r#"
        f = lambda x: x + 1
        assert_eq(f(1), 2)

        g = lambda x, y: x * y
        assert_eq(g(3, 4), 12)

        assert_eq((lambda x: x*x)(5), 25)
    "#,
    );
}

#[test]
fn test_function_errors() {
    assert::fail("1()", "Cannot call value");
    assert::fail("def f(x): pass; f()", "Missing required argument");
    assert::fail("def f(): pass; f(1)", "too many positional arguments");
    assert::fail("def f(): pass; f(a=1)", "unexpected keyword arguments");
}
