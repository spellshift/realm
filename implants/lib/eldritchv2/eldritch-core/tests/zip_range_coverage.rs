mod assert;

#[test]
fn test_zip_empty() {
    assert::pass(
        r#"
        assert_eq(zip(), [])
    "#,
    );
}

#[test]
fn test_zip_iterables() {
    assert::pass(
        r#"
        # Lists
        assert_eq(zip([1, 2], [3, 4]), [(1, 3), (2, 4)])

        # Tuples
        assert_eq(zip((1, 2), (3, 4)), [(1, 3), (2, 4)])

        # Strings
        assert_eq(zip("ab", "cd"), [("a", "c"), ("b", "d")])

        # Mixed
        assert_eq(zip([1, 2], "ab"), [(1, "a"), (2, "b")])
    "#,
    );
}

#[test]
fn test_zip_length_mismatch() {
    assert::pass(
        r#"
        # Shortest list wins
        assert_eq(zip([1, 2, 3], [4, 5]), [(1, 4), (2, 5)])
        assert_eq(zip([1, 2], [3, 4, 5]), [(1, 3), (2, 4)])
    "#,
    );
}

#[test]
fn test_zip_errors() {
    assert::fail("zip(1)", "not iterable");
    assert::fail("zip([1], 1)", "not iterable");
}

#[test]
fn test_range_step_zero() {
    assert::fail("range(0, 10, 0)", "range() arg 3 must not be zero");
}

#[test]
fn test_range_large_step() {
    assert::pass(
        r#"
        assert_eq(range(0, 10, 100), [0])
        assert_eq(range(0, 10, 10), [0])
        assert_eq(range(0, 10, 11), [0])
    "#,
    );
}

#[test]
fn test_range_arguments() {
    // 1 arg
    assert::pass(
        r#"
        assert_eq(range(5), [0, 1, 2, 3, 4])
    "#,
    );

    // 2 args
    assert::pass(
        r#"
        assert_eq(range(1, 5), [1, 2, 3, 4])
    "#,
    );

    // 3 args
    assert::pass(
        r#"
        assert_eq(range(0, 10, 2), [0, 2, 4, 6, 8])
    "#,
    );

    // Negative step
    assert::pass(
        r#"
        assert_eq(range(5, 0, -1), [5, 4, 3, 2, 1])
    "#,
    );
}

#[test]
fn test_range_errors() {
    assert::fail("range()", "range expects 1-3 integer arguments");
    assert::fail("range(1, 2, 3, 4)", "range expects 1-3 integer arguments");
    assert::fail("range('a')", "range expects 1-3 integer arguments");
}
