mod assert;

#[test]
fn test_mixed_arithmetic_ops_extended() {
    // Missing mixed types coverage
    assert::pass(
        r#"
    # Float + Int
    assert_eq(2.0 + 1, 3.0)

    # Int - Float
    assert_eq(3 - 1.0, 2.0)

    # Float * Int
    assert_eq(3.0 * 2, 6.0)

    # Int / Float
    assert_eq(6 / 2.0, 3.0)

    # Float // Int
    assert_eq(6.0 // 2, 3.0)
    assert_eq(7.5 // 2, 3.0)

    # Int % Float
    assert_eq(7 % 3.0, 1.0)
    assert_eq(7 % 2.5, 2.0)
    "#,
    );
}

#[test]
fn test_arithmetic_type_errors() {
    // Unsupported operands
    assert::fail(
        "1 + \"a\"",
        "unsupported operand type(s) for +: 'int' and 'string'",
    );
    assert::fail(
        "\"a\" - 1",
        "unsupported operand type(s) for -: 'string' and 'int'",
    );
    assert::fail(
        "[1] / 2",
        "unsupported operand type(s) for /: 'list' and 'int'",
    );
    assert::fail(
        "1 // [2]",
        "unsupported operand type(s) for //: 'int' and 'list'",
    );
    assert::fail(
        "1 % [2]",
        "unsupported operand type(s) for %: 'int' and 'list'",
    );
    assert::fail(
        "1.0 + \"a\"",
        "unsupported operand type(s) for +: 'float' and 'string'",
    );
}
