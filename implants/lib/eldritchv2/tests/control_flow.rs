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
