mod assert;

#[test]
fn test_string_formatting_decimal() {
    assert::pass(
        r#"
        assert_eq("%d" % 10, "10")
        assert_eq("%d" % -10, "-10")
        assert_eq("%i" % 10, "10")
        assert_eq("%u" % 10, "10")
        # Floats are truncated
        assert_eq("%d" % 10.5, "10")
        # Bools
        assert_eq("%d" % True, "1")
        assert_eq("%d" % False, "0")
    "#,
    );
}

#[test]
fn test_string_formatting_hex_octal() {
    assert::pass(
        r#"
        assert_eq("%x" % 10, "a")
        assert_eq("%X" % 10, "A")
        assert_eq("%o" % 10, "12")
        # Floats
        assert_eq("%x" % 15.5, "f")
    "#,
    );
}

#[test]
fn test_string_formatting_float() {
    // Default precision is .6
    assert::pass(
        r#"
        assert_eq("%f" % 1.5, "1.500000")
        assert_eq("%F" % 1.5, "1.500000")
        # Ints converted to float
        assert_eq("%f" % 1, "1.000000")

        # Scientific
        # Exact representation depends on implementation but should contain 'e'
        s = "%e" % 1000.0
        assert("e" in s)
    "#,
    );
}

#[test]
fn test_string_formatting_repr() {
    assert::pass(
        r#"
        assert_eq("%r" % "hello", "\"hello\"")
        assert_eq("%r" % 10, "10")
        assert_eq("%r" % [1, 2], "[1, 2]")
    "#,
    );
}

#[test]
fn test_string_formatting_string() {
    assert::pass(
        r#"
        assert_eq("%s" % "hello", "hello")
        assert_eq("%s" % 10, "10")
        assert_eq("%s" % [1, 2], "[1, 2]")
    "#,
    );
}

#[test]
fn test_string_formatting_literals() {
    assert::pass(
        r#"
        assert_eq("%%" % (), "%")
        assert_eq("100%%" % (), "100%")
        assert_eq("Value: %d%%" % 50, "Value: 50%")
    "#,
    );
}

#[test]
fn test_string_formatting_tuple_args() {
    assert::pass(
        r#"
        assert_eq("%d %d" % (1, 2), "1 2")
        # Single tuple as argument needs to be wrapped? No, in Python (1, 2) is the args.
        # But if we want to format a tuple itself with %s?
        # "%s" % (1, 2) -> TypeError not all arguments converted
        # "%s" % ((1, 2),) -> "(1, 2)"

        t = (1, 2)
        assert_eq("%s" % (t,), "(1, 2)")
    "#,
    );
}

#[test]
fn test_string_formatting_errors() {
    // Not enough args
    assert::fail("'%s %s' % (1,)", "not enough arguments");

    // Too many args
    assert::fail("'%s' % (1, 2)", "not all arguments converted");

    // Invalid type for specifier
    assert::fail("'%d' % 'a'", "number is required");
    assert::fail("'%d' % []", "number is required");

    // Incomplete format
    assert::fail("'%' % ()", "incomplete format key");

    // Unsupported char
    assert::fail("'%q' % 1", "unsupported format character 'q'");
}
