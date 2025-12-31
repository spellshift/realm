mod assert;

#[test]
fn test_enumerate_list() {
    assert::pass(
        r#"
        e = enumerate(['a', 'b'])
        assert_eq(e, [(0, 'a'), (1, 'b')])
    "#,
    );
}

#[test]
fn test_enumerate_start() {
    assert::pass(
        r#"
        e = enumerate(['a', 'b'], 10)
        assert_eq(e, [(10, 'a'), (11, 'b')])
    "#,
    );
}

#[test]
fn test_enumerate_string() {
    assert::pass(
        r#"
        e = enumerate("ab")
        assert_eq(e, [(0, "a"), (1, "b")])
    "#,
    );
}

#[test]
fn test_enumerate_empty() {
    assert::pass(
        r#"
        assert_eq(enumerate([]), [])
        assert_eq(enumerate(""), [])
    "#,
    );
}

#[test]
fn test_enumerate_errors() {
    assert::fail("enumerate()", "enumerate() takes at least one argument");
    assert::fail(
        "enumerate([1], 'a')",
        "enumerate() start must be an integer",
    );
    assert::fail("enumerate(1)", "Type '\"int\"' is not iterable");
}

#[test]
fn test_enumerate_set() {
    // Sets are unordered, so we check existence of tuples
    assert::pass(
        r#"
        s = {1}
        e = enumerate(s)
        assert_eq(len(e), 1)
        assert_eq(e[0][0], 0)
        assert_eq(e[0][1], 1)
    "#,
    );
}
