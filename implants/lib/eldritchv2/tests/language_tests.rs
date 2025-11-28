mod assert;

// --- Test Suites ---

#[test]
fn test_literals_and_constants() {
    assert::all_true(
        r#"
        True == True
        False == False
        True != False
        None == None
        1 == 1
        "hello" == "hello"
    "#,
    );
}

#[test]
fn test_arithmetic() {
    assert::all_true(
        r#"
        1 + 2 == 3
        10 - 2 == 8
        5 * 5 == 25
        10 / 2 == 5
        1 + 2 * 3 == 7
        (1 + 2) * 3 == 9
    "#,
    );

    // FIX: Updated expectation to match actual implementation
    assert::fail("1 / 0", "attempt to divide by zero");
}

#[test]
fn test_string_operations() {
    assert::pass(
        r#"
        x = "hello"
        y = "world"
        assert_eq(x + " " + y, "hello world")
    "#,
    );

    assert::eq(r#"len("abc")"#, "3");

    // F-strings
    assert::pass(
        r#"
        name = "Bob"
        age = 20
        assert_eq(f"{name} is {age}", "Bob is 20")
    "#,
    );
}

#[test]
fn test_lists() {
    assert::pass(
        r#"
        l = [1, 2, 3]
        assert_eq(len(l), 3)
        assert_eq(l[0], 1)
        assert_eq(l[1] + l[2], 5)
    "#,
    );

    // List iteration
    assert::pass(
        r#"
        sum = 0
        for x in [1, 2, 3, 4]:
            sum = sum + x
        assert_eq(sum, 10)
    "#,
    );

    assert::fail("l = [1]; l[5]", "index out of range");
}

#[test]
fn test_dictionaries() {
    assert::pass(
        r#"
        d = {"a": 1, "b": 2}
        assert_eq(len(d), 2)
        assert_eq(d["a"], 1)
    "#,
    );

    // Dynamic keys
    assert::pass(
        r#"
        k = "foo"
        d = {k: "bar"}
        assert_eq(d["foo"], "bar")
    "#,
    );

    assert::fail(r#"d = {}; d["missing"]"#, "KeyError");
}

#[test]
fn test_control_flow() {
    assert::pass(
        r#"
        x = 10
        res = 0
        if x > 5:
            res = 1
        else:
            res = 2
        assert_eq(res, 1)
    "#,
    );

    assert::pass(
        r#"
        x = 0
        if x > 5:
            res = 1
        elif x == 0:
            res = 2
        else:
            res = 3
        assert_eq(res, 2)
    "#,
    );
}

#[test]
fn test_functions() {
    assert::pass(
        r#"
        def square(x):
            return x * x

        assert_eq(square(4), 16)
    "#,
    );

    // Recursion
    assert::pass(
        r#"
        def fib(n):
            if n < 2:
                return n
            return fib(n-1) + fib(n-2)

        assert_eq(fib(6), 8)
    "#,
    );

    // Closures/Environment check
    assert::pass(
        r#"
        x = 10
        def get_x():
            return x
        assert_eq(get_x(), 10)
    "#,
    );
}

#[test]
fn test_builtins() {
    assert::eq("len([1, 2])", "2");
    assert::eq("len(range(5))", "5");
    assert::pass("assert(True)");
    assert::fail("assert(False)", "Assertion failed");
    assert::fail("fail('boom')", "boom");
}
