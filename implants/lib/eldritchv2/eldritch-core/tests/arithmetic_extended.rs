mod assert;

#[test]
fn test_mixed_type_errors() {
    assert::fail("1 + 'a'", "unsupported operand type(s) for +: 'int' and 'string'");
    assert::fail("'a' + 1", "unsupported operand type(s) for +: 'string' and 'int'");
    assert::fail("1 - 'a'", "unsupported operand type(s) for -: 'int' and 'string'");
    assert::fail("'a' - 1", "unsupported operand type(s) for -: 'string' and 'int'");
    // 1 * 'a' is valid (repetition), resulting in "a"
    assert::pass("assert_eq(1 * 'a', 'a')");
    assert::fail("1 / 'a'", "unsupported operand type(s) for /: 'int' and 'string'");
    assert::fail("'a' / 1", "unsupported operand type(s) for /: 'string' and 'int'");
    assert::fail("1 // 'a'", "unsupported operand type(s) for //: 'int' and 'string'");
    assert::fail("'a' // 1", "unsupported operand type(s) for //: 'string' and 'int'");
    assert::fail("1 % 'a'", "unsupported operand type(s) for %: 'int' and 'string'");
    // Note: 'a' % 1 might be valid (string formatting), but 'a' % 'b' is not usually unless 'a' has format specifiers and 'b' is the arg
    assert::fail("'a' % []", "not all arguments converted during string formatting"); // "a" has 0 specifiers, [] is 1 arg.
    // Actually strict string formatting tests are elsewhere, but let's test invalid types for math ops

    assert::fail("1 + []", "unsupported operand type(s) for +: 'int' and 'list'");
    assert::fail("[] + 1", "unsupported operand type(s) for +: 'list' and 'int'");
}

#[test]
fn test_extended_floor_div_mod() {
    // Int // Int
    assert::pass("assert_eq(10 // 3, 3)");
    assert::pass("assert_eq(-10 // 3, -4)");
    assert::pass("assert_eq(10 // -3, -4)");
    assert::pass("assert_eq(-10 // -3, 3)");

    // Float // Float
    assert::pass("assert_eq(10.0 // 3.0, 3.0)");
    assert::pass("assert_eq(-10.0 // 3.0, -4.0)");
    assert::pass("assert_eq(10.0 // -3.0, -4.0)");
    assert::pass("assert_eq(-10.0 // -3.0, 3.0)");

    // Mixed Int // Float
    assert::pass("assert_eq(10 // 3.0, 3.0)");
    assert::pass("assert_eq(-10 // 3.0, -4.0)");
    assert::pass("assert_eq(10 // -3.0, -4.0)");
    assert::pass("assert_eq(-10 // -3.0, 3.0)");

    // Mixed Float // Int
    assert::pass("assert_eq(10.0 // 3, 3.0)");
    assert::pass("assert_eq(-10.0 // 3, -4.0)");
    assert::pass("assert_eq(10.0 // -3, -4.0)");
    assert::pass("assert_eq(-10.0 // -3, 3.0)");

    // Int % Int
    assert::pass("assert_eq(10 % 3, 1)");
    assert::pass("assert_eq(-10 % 3, 2)");
    assert::pass("assert_eq(10 % -3, -2)");
    assert::pass("assert_eq(-10 % -3, -1)");

    // Float % Float
    // 10.0 = 3.0 * 3.0 + 1.0
    assert::pass("assert_eq(10.0 % 3.0, 1.0)");
    // -10.0 = 3.0 * -4.0 + 2.0
    assert::pass("assert_eq(-10.0 % 3.0, 2.0)");
    // 10.0 = -3.0 * -4.0 + -2.0
    assert::pass("assert_eq(10.0 % -3.0, -2.0)");
    // -10.0 = -3.0 * 3.0 + -1.0
    assert::pass("assert_eq(-10.0 % -3.0, -1.0)");
}

#[test]
fn test_bitwise_invalid_types() {
    assert::fail("1.0 & 1", "unsupported operand type(s) for &: 'float' and 'int'");
    assert::fail("1 & 1.0", "unsupported operand type(s) for &: 'int' and 'float'");
    assert::fail("'a' & 1", "unsupported operand type(s) for &: 'string' and 'int'");

    assert::fail("1.0 | 1", "unsupported operand type(s) for |: 'float' and 'int'");
    assert::fail("1 | 1.0", "unsupported operand type(s) for |: 'int' and 'float'");

    assert::fail("1.0 ^ 1", "unsupported operand type(s) for ^: 'float' and 'int'");
    assert::fail("1 ^ 1.0", "unsupported operand type(s) for ^: 'int' and 'float'");

    assert::fail("1.0 << 1", "unsupported operand type(s) for <<: 'float' and 'int'");
    assert::fail("1 << 1.0", "unsupported operand type(s) for <<: 'int' and 'float'");

    assert::fail("1.0 >> 1", "unsupported operand type(s) for >>: 'float' and 'int'");
    assert::fail("1 >> 1.0", "unsupported operand type(s) for >>: 'int' and 'float'");
}
