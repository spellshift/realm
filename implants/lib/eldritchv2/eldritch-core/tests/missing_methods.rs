mod assert;

#[test]
fn test_dict_missing_methods() {
    // Test clear()
    assert::pass(
        r#"
        d = {"a": 1, "b": 2}
        d.clear()
        assert_eq(len(d), 0)
        assert_eq(d, {})
        d.clear() # Should be fine on empty dict
        assert_eq(len(d), 0)
    "#,
    );

    // Test pop()
    assert::pass(
        r#"
        d = {"a": 1, "b": 2}
        val = d.pop("a")
        assert_eq(val, 1)
        assert_eq(d, {"b": 2})

        # Pop with default
        val = d.pop("z", 999)
        assert_eq(val, 999)
        assert_eq(d, {"b": 2}) # Should not modify
    "#,
    );

    // Test pop() errors
    assert::fail(
        "d = {}; d.pop('a')",
        "KeyError: a"
    );
    assert::fail(
        "d = {'a': 1}; d.pop()",
        "TypeError: pop() takes between 1 and 2 arguments"
    );

    // Test setdefault()
    assert::pass(
        r#"
        d = {"a": 1}
        # Existing key
        val = d.setdefault("a", 2)
        assert_eq(val, 1)
        assert_eq(d["a"], 1)

        # New key with default
        val = d.setdefault("b", 2)
        assert_eq(val, 2)
        assert_eq(d["b"], 2)

        # New key without default (should be None)
        val = d.setdefault("c")
        assert_eq(val, None)
        assert_eq(d["c"], None)
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
