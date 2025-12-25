mod assert;

#[test]
fn test_range_one_arg() {
    assert::pass(
        r#"
        r = range(5)
        assert_eq(r, [0, 1, 2, 3, 4])
        assert_eq(len(r), 5)
    "#,
    );
}

#[test]
fn test_range_two_args() {
    assert::pass(
        r#"
        r = range(1, 6)
        assert_eq(r, [1, 2, 3, 4, 5])
        assert_eq(len(r), 5)
    "#,
    );
}

#[test]
fn test_range_three_args() {
    assert::pass(
        r#"
        r = range(0, 10, 2)
        assert_eq(r, [0, 2, 4, 6, 8])
        assert_eq(len(r), 5)
    "#,
    );
}

#[test]
fn test_range_negative_step() {
    assert::pass(
        r#"
        r = range(5, 0, -1)
        assert_eq(r, [5, 4, 3, 2, 1])
        assert_eq(len(r), 5)
    "#,
    );
}

#[test]
fn test_range_zero_step() {
    assert::fail(
        r#"
        range(0, 10, 0)
    "#,
        "ValueError: range() arg 3 must not be zero",
    );
}

#[test]
fn test_range_empty() {
    assert::pass(
        r#"
        r = range(0)
        assert_eq(r, [])
        r = range(10, 0)
        assert_eq(r, [])
        r = range(0, 10, -1)
        assert_eq(r, [])
    "#,
    );
}

#[test]
fn test_range_types() {
    assert::fail("range('a')", "TypeError: range expects 1-3 integer arguments");
    assert::fail("range(1, 'b')", "TypeError: range expects 1-3 integer arguments");
    assert::fail("range(1, 2, 'c')", "TypeError: range expects 1-3 integer arguments");
    assert::fail("range()", "TypeError: range expects 1-3 integer arguments");
    assert::fail("range(1, 2, 3, 4)", "TypeError: range expects 1-3 integer arguments");
}
