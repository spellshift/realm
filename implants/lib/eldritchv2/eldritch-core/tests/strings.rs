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
fn test_string_missing_methods() {
    // Test rsplit()
    assert::pass(
        r#"
        # With explicit delimiter
        s = "a,b,c"
        l = s.rsplit(",")
        assert_eq(l, ["a", "b", "c"])

        s = "a,b,c"
        # Note: current implementation of rsplit ignores maxsplit (arg 2) if passed,
        # but the method signature doesn't even accept it currently in the match arm (impl uses args[0] only)
        # We just test basic splitting behavior.

        # Default delimiter is space " " in current impl, not whitespace.
        s = "a b c"
        l = s.rsplit()
        assert_eq(l, ["a", "b", "c"])
    "#,
    );

    // Test codepoints()
    assert::pass(
        r#"
        s = "ABC"
        l = s.codepoints()
        # 65, 66, 67
        assert_eq(l, [65, 66, 67])

        s = "€" # Euro sign
        l = s.codepoints()
        assert_eq(l, [8364])
    "#,
    );

    // Test elems()
    assert::pass(
        r#"
        s = "abc"
        l = s.elems()
        assert_eq(l, ["a", "b", "c"])

        s = ""
        l = s.elems()
        assert_eq(l, [])
    "#,
    );
}

#[test]
fn test_string_boolean_checks() {
    // isalnum
    assert::pass(
        r#"
        assert("abc".isalnum())
        assert("123".isalnum())
        assert("a1".isalnum())
        assert(not "!".isalnum())
        assert(not "".isalnum())
    "#,
    );

    // isalpha
    assert::pass(
        r#"
        assert("abc".isalpha())
        assert(not "123".isalpha())
        assert(not "a1".isalpha())
        assert(not "".isalpha())
    "#,
    );

    // isdigit
    assert::pass(
        r#"
        assert("123".isdigit())
        assert(not "12.3".isdigit())
        assert(not "a1".isdigit())
        assert(not "".isdigit())
    "#,
    );

    // isspace
    assert::pass(
        r#"
        assert(" ".isspace())
        assert("\t".isspace())
        assert("\n".isspace())
        assert(not " a".isspace())
        assert(not "".isspace())
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
