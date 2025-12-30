mod assert;

#[test]
fn test_mixed_comparisons_extended() {
    // Int vs Float comparisons
    assert::pass(
        r#"
    # Equality
    assert_eq(1 == 1.0, true)
    assert_eq(1.0 == 1, true)
    assert_eq(1 == 1.1, false)
    assert_eq(1.1 == 1, false)

    # Inequality
    assert_eq(1 != 1.0, false)
    assert_eq(1.0 != 1, false)
    assert_eq(1 != 1.1, true)
    assert_eq(1.1 != 1, true)

    # Less than
    assert_eq(1 < 1.1, true)
    assert_eq(1.1 < 2, true)
    assert_eq(2 < 1.1, false)

    # Greater than
    assert_eq(1.1 > 1, true)
    assert_eq(2 > 1.1, true)
    assert_eq(1 > 1.1, false)

    # Less equal
    assert_eq(1 <= 1.0, true)
    assert_eq(1 <= 1.1, true)
    assert_eq(1.1 <= 1, false)

    # Greater equal
    assert_eq(1.0 >= 1, true)
    assert_eq(1.1 >= 1, true)
    assert_eq(0.9 >= 1, false)
    "#,
    );
}

#[test]
fn test_comparison_type_errors() {
    // Mismatched types
    assert::fail("1 < \"a\"", "'<' not supported between instances of 'int' and 'string'");
    assert::fail("\"a\" > 1", "'>' not supported between instances of 'string' and 'int'");
    assert::fail("[1] <= 1", "'<=' not supported between instances of 'list' and 'int'");
    assert::fail("1 >= [1]", "'>=' not supported between instances of 'int' and 'list'");
}
