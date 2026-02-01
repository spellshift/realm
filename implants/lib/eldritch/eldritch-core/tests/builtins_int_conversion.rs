mod assert;

#[test]
fn test_int_conversion_success() {
    // Basic base 10
    assert::pass("assert_eq(int('5'), 5)");
    assert::pass("assert_eq(int('10'), 10)");
    assert::pass("assert_eq(int('-5'), -5)");

    // Explicit base 16
    assert::pass("assert_eq(int('0x0a', 16), 10)");
    assert::pass("assert_eq(int('10', 16), 16)");
    assert::pass("assert_eq(int('0a', 16), 10)");
    assert::pass("assert_eq(int('0A', 16), 10)");
    assert::pass("assert_eq(int('-0x0a', 16), -10)");

    // Base 0 (auto-detect)
    assert::pass("assert_eq(int('0x10', 0), 16)");
    assert::pass("assert_eq(int('0o10', 0), 8)");
    assert::pass("assert_eq(int('0b10', 0), 2)");
    assert::pass("assert_eq(int('10', 0), 10)");

    // Other bases
    assert::pass("assert_eq(int('10', 2), 2)");
    assert::pass("assert_eq(int('10', 8), 8)");

    // Float conversion
    assert::pass("assert_eq(int(5.2), 5)");
    assert::pass("assert_eq(int(5.9), 5)");
    assert::pass("assert_eq(int(-5.2), -5)");

    // Edge cases (i64 limits)
    assert::pass("assert_eq(int('9223372036854775807'), 9223372036854775807)");
    // Note: -9223372036854775808 literal might be parsed as -(9223372036854775808) which overflows i64 positive,
    // so we use subtraction to represent i64::MIN safely in the test assertion.
    assert::pass("assert_eq(int('-9223372036854775808'), -9223372036854775807 - 1)");
}

#[test]
fn test_int_conversion_errors() {
    // Invalid literal for base 10 (default)
    assert::fail("int('0a')", "invalid literal for int() with base 10: '0a'");

    // Invalid literal for explicit base
    assert::fail(
        "int('g', 16)",
        "invalid literal for int() with base 16: 'g'",
    );

    // Non-string with explicit base
    assert::fail(
        "int(5, 16)",
        "int() can't convert non-string with explicit base",
    );

    // Base out of range (if we support 2-36)
    // Python raises ValueError: int() base must be >= 2 and <= 36, or 0
    assert::fail("int('1', 1)", "int() base must be >= 2 and <= 36, or 0");
    assert::fail("int('1', 37)", "int() base must be >= 2 and <= 36, or 0");
}
