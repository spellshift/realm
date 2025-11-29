mod assert;

#[test]
fn test_string_literals() {
    assert::pass(
        r#"
        x = "hello"
        y = "world"
        assert_eq(x + " " + y, "hello world")
    "#,
    );
}

#[test]
fn test_f_strings() {
    assert::pass(
        r#"
        name = "Bob"
        age = 20
        assert_eq(f"{name} is {age}", "Bob is 20")
        assert_eq(f"Next is {age + 1}", "Next is 21")
    "#,
    );
}

#[test]
fn test_byte_strings() {
    assert::pass(
        r#"
        b = b"hello"
        assert_eq(len(b), 5)
        assert_eq(type(b), "bytes")
    "#,
    );
}

#[test]
fn test_doc_strings() {
    assert::pass(
        r#"
        def func():
            """This is a docstring"""
            return 1
        assert_eq(func(), 1)
    "#,
    );

    assert::pass(
        r#"
        s = """line1
        line2"""
        assert_eq(type(s), "string")
        # Check content if possible, or just compilation
    "#,
    );
}

#[test]
fn test_string_methods() {
    assert::pass(
        r#"
        s = "Hello World"
        assert_eq(s.lower(), "hello world")
        assert_eq(s.upper(), "HELLO WORLD")
        assert_eq(" a ".strip(), "a")
        assert_eq("a,b".split(","), ["a", "b"])
        assert_eq("a".startswith("a"), True)
        assert_eq("a".endswith("z"), False)
        assert_eq("abc".find("b"), 1)
        assert_eq("aba".replace("a", "o"), "obo")
        assert_eq("-".join(["a", "b"]), "a-b")
        assert_eq("Hi {}".format("There"), "Hi There")
    "#,
    );
}

#[test]
fn test_string_slicing() {
    assert::pass(
        r#"
        s = "abcdef"
        assert_eq(s[::-1], "fedcba")
        assert_eq(s[1:4], "bcd")
        assert_eq(s[4:1:-1], "edc")
    "#,
    );
}
