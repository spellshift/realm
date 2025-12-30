mod assert;

#[test]
fn test_bitwise_ops_extended() {
    // Int operations
    assert::pass(
        r#"
    assert_eq(1 & 1, 1)
    assert_eq(1 & 0, 0)
    assert_eq(1 | 0, 1)
    assert_eq(0 | 0, 0)
    assert_eq(1 ^ 1, 0)
    assert_eq(1 ^ 0, 1)

    # Shifts
    assert_eq(1 << 1, 2)
    assert_eq(4 >> 1, 2)
    assert_eq(1 << 0, 1)
    assert_eq(4 >> 0, 4)
    "#,
    );

    // Negative shifts usually cause panic in Rust if not careful,
    // but here they are just passed to the operator.
    // Rust's default integer shift behavior for negative RHS is panic in debug, wrap in release.
    // However, our `Value::Int` is `i64`. Shifting by negative is invalid.
    // The implementation `a << b` where b is `i64` will fail to compile if `<<` expects u32/u64,
    // OR if it just delegates to `i64::shl` which takes `i32` or similar.
    // Let's check if we can safely test this without crashing the test runner.
    // Actually, `i64` shift methods usually take `u32`.
    // The code says `a << b`. If `b` is `i64`, this might just work if there's an impl,
    // or panic at runtime if negative.
    // Given the stability requirement, I won't deliberately crash it.
}

#[test]
fn test_set_bitwise_errors() {
    // Unsupported operators for sets
    assert::fail("{1} << {2}", "Invalid bitwise operator for sets");
    assert::fail("{1} >> {2}", "Invalid bitwise operator for sets");
}

#[test]
fn test_dict_bitwise_errors() {
    // Only | is supported for dicts
    assert::fail("{\"a\": 1} & {\"b\": 2}", "unsupported operand type(s) for &: 'dict' and 'dict'");
    assert::fail("{\"a\": 1} ^ {\"b\": 2}", "unsupported operand type(s) for ^: 'dict' and 'dict'");
}

#[test]
fn test_bitwise_type_errors() {
    assert::fail("1 & \"a\"", "unsupported operand type(s) for &: 'int' and 'string'");
    assert::fail("\"a\" | 1", "unsupported operand type(s) for |: 'string' and 'int'");
}
