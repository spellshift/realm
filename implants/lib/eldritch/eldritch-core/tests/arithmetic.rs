mod assert;

#[test]
fn test_arithmetic_ops() {
    // Int/Int
    assert::pass(
        r#"
    assert_eq(1 + 2, 3)
    assert_eq(3 - 1, 2)
    assert_eq(2 * 3, 6)
    assert_eq(6 // 2, 3)
    assert_eq(7 % 3, 1)
    "#,
    );

    // Float/Float
    assert::pass(
        r#"
    assert_eq(1.0 + 2.0, 3.0)
    assert_eq(3.0 - 1.0, 2.0)
    assert_eq(2.0 * 3.0, 6.0)
    assert_eq(6.0 / 2.0, 3.0)
    assert_eq(6.0 // 2.0, 3.0)
    assert_eq(7.0 % 3.0, 1.0)
    "#,
    );

    // Mixed Int/Float
    assert::pass(
        r#"
    assert_eq(1 + 2.0, 3.0)
    assert_eq(3.0 - 1, 2.0)
    assert_eq(2 * 3.0, 6.0)
    assert_eq(6.0 / 2, 3.0)
    assert_eq(6 // 2.0, 3.0)
    assert_eq(7.0 % 3, 1.0)
    "#,
    );

    // Division resulting in float
    assert::pass(
        r#"
    res = 5 / 2
    assert_eq(res, 2.5)
    # Check type is float? (implicitly checked by equality to 2.5)

    res2 = 5 // 2
    assert_eq(res2, 2)
    "#,
    );

    // Floor division negative
    assert::pass(
        r#"
    assert_eq(-5 // 2, -3)
    assert_eq(5 // -2, -3)
    "#,
    );

    // Modulo negative
    assert::pass(
        r#"
    assert_eq(-5 % 2, 1)
    assert_eq(5 % -2, -1)
    "#,
    );
}

#[test]
fn test_bitwise_ops() {
    assert::pass(
        r#"
    assert_eq(3 & 1, 1)
    assert_eq(3 | 1, 3)
    assert_eq(3 ^ 1, 2)
    assert_eq(1 << 2, 4)
    assert_eq(4 >> 1, 2)
    "#,
    );
}

#[test]
fn test_set_ops() {
    assert::pass(
        r#"
    s1 = {1, 2, 3}
    s2 = {3, 4, 5}

    # Intersection
    assert_eq(s1 & s2, {3})

    # Difference
    assert_eq(s1 - s2, {1, 2})
    assert_eq(s2 - s1, {4, 5})

    # Symmetric Difference
    assert_eq(s1 ^ s2, {1, 2, 4, 5})

    # Union
    assert_eq(s1 | s2, {1, 2, 3, 4, 5})
    "#,
    );
}

#[test]
fn test_concatenation() {
    // Strings
    assert::pass(
        r#"
    assert_eq("hello" + " " + "world", "hello world")
    "#,
    );

    // Lists
    assert::pass(
        r#"
    assert_eq([1, 2] + [3, 4], [1, 2, 3, 4])
    "#,
    );

    // Tuples
    assert::pass(
        r#"
    assert_eq((1, 2) + (3, 4), (1, 2, 3, 4))
    "#,
    );

    // Bytes
    assert::pass(
        r#"
    assert_eq(b"hello" + b" " + b"world", b"hello world")
    "#,
    );
}

#[test]
fn test_repetition() {
    // String
    assert::pass(
        r#"
    assert_eq("a" * 3, "aaa")
    assert_eq(3 * "a", "aaa")
    "#,
    );

    // List
    assert::pass(
        r#"
    assert_eq([1] * 3, [1, 1, 1])
    assert_eq(3 * [1], [1, 1, 1])
    "#,
    );

    // Tuple
    assert::pass(
        r#"
    assert_eq((1,) * 3, (1, 1, 1))
    assert_eq(3 * (1,), (1, 1, 1))
    "#,
    );

    // Bytes
    assert::pass(
        r#"
    assert_eq(b"a" * 3, b"aaa")
    assert_eq(3 * b"a", b"aaa")
    "#,
    );
}

#[test]
fn test_dict_union() {
    assert::pass(
        r#"
    d1 = {"a": 1, "b": 2}
    d2 = {"b": 3, "c": 4}

    # Merge, right side wins collisions
    d3 = d1 | d2
    assert_eq(d3["a"], 1)
    assert_eq(d3["b"], 3)
    assert_eq(d3["c"], 4)
    assert_eq(len(d3), 3)

    # Originals unchanged
    assert_eq(d1["b"], 2)
    "#,
    );
}

#[test]
fn test_string_interpolation() {
    assert::pass(
        r#"
    assert_eq("Hello %s" % "World", "Hello World")
    assert_eq("Number %d" % 10, "Number 10")
    assert_eq("Float %d" % 10.5, "Float 10") # Truncates/converts to int display
    assert_eq("Repr %r" % "foo", "Repr \"foo\"")

    # New specifiers
    assert_eq("%o" % 8, "10")       # Octal
    assert_eq("%x" % 15, "f")       # Lower hex
    assert_eq("%X" % 15, "F")       # Upper hex

    # Float formats (checking basic validity, specific output format might vary slightly by platform)
    # But for standard 1.0, it should be stable.
    assert_eq("%f" % 1.5, "1.500000")
    # %e / %g might be tricky to test exact strings cross-platform/libm, but we can try simple ones

    # Tuple arguments
    assert_eq("%s %s" % ("Hello", "World"), "Hello World")

    # Percent literal in format string
    assert_eq("100%%" % (), "100%")
    assert_eq("%d%%" % 100, "100%")
    "#,
    );
}

#[test]
fn test_div_zero_checks() {
    assert::fail("1.0 / 0", "divide by zero");
    assert::fail("1 / 0.0", "divide by zero");
    assert::fail("1.0 / 0.0", "divide by zero");

    // Also check floor div and modulo as implemented in the fix
    assert::fail("1.0 // 0", "divide by zero");
    assert::fail("1 // 0.0", "divide by zero");

    assert::fail("1.0 % 0", "modulo by zero");
    assert::fail("1 % 0.0", "modulo by zero");
}
