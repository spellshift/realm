mod assert;

#[test]
fn test_reversed_list() {
    assert::pass(
        r#"
        r = reversed([1, 2, 3])
        assert_eq(r, [3, 2, 1])
    "#,
    );
}

#[test]
fn test_reversed_tuple() {
    assert::pass(
        r#"
        r = reversed((1, 2, 3))
        assert_eq(r, [3, 2, 1])
    "#,
    );
}

#[test]
fn test_reversed_string() {
    assert::pass(
        r#"
        r = reversed("abc")
        assert_eq(r, ["c", "b", "a"])
    "#,
    );
}

#[test]
fn test_reversed_empty() {
    assert::pass(
        r#"
        assert_eq(reversed([]), [])
        assert_eq(reversed(()), [])
        assert_eq(reversed(""), [])
    "#,
    );
}

#[test]
fn test_reversed_errors() {
    assert::fail("reversed()", "reversed() takes exactly one argument (0 given)");
    assert::fail(
        "reversed(1, 2)",
        "reversed() takes exactly one argument (2 given)",
    );
    assert::fail("reversed(1)", "'int' object is not reversible");
    assert::fail("reversed({'a': 1})", "'dict' object is not reversible");
    assert::fail("reversed({1, 2})", "'set' object is not reversible");
}
