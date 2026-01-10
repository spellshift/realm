mod assert;

#[test]
fn test_exponentiation() {
    // Int ** Int (Simple)
    assert::pass(
        r#"
        assert_eq(2 ** 3, 8)
        assert_eq(3 ** 2, 9)
        assert_eq(10 ** 0, 1)
        assert_eq(0 ** 0, 1) # Standard mathematical convention
        assert_eq(0 ** 5, 0)
    "#,
    );

    // Int ** Int (Negative exponent -> Float)
    assert::pass(
        r#"
        res = 2 ** -1
        assert_eq(res, 0.5)
        # Check type logic indirectly or simply trust equality
        # 10 ** -2 = 0.01
        res2 = 10 ** -2
        assert_eq(res2, 0.01)
    "#,
    );

    // Float ** Float
    assert::pass(
        r#"
        assert_eq(2.0 ** 3.0, 8.0)
        assert_eq(4.0 ** 0.5, 2.0) # Square root
    "#,
    );

    // Mixed Types
    assert::pass(
        r#"
        assert_eq(2 ** 3.0, 8.0)
        assert_eq(2.0 ** 3, 8.0)
        assert_eq(9 ** 0.5, 3.0)
    "#,
    );

    // Overflow behavior (should degrade to float instead of panicking)
    assert::pass(
        r#"
        # 2^62 fits in i64 (max is 2^63 - 1)
        # 2^63 overflows positive i64
        val = 2 ** 63
        # Should be a float approx 9.22e18
        # We can't easily check type without `type()` built-in which returns string
        # But we can check it is positive
        assert(val > 0)
        assert_eq(type(val), "float")
    "#,
    );
}
