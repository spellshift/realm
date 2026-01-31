mod assert;

#[test]
fn test_splitlines_extended() {
    assert::pass(
        r#"
        # Default behavior (keepends=False)
        assert_eq("a\nb\nc".splitlines(), ["a", "b", "c"])
        assert_eq("a\nb\nc\n".splitlines(), ["a", "b", "c"])

        # With keepends=True
        assert_eq("a\nb".splitlines(True), ["a\n", "b"])
        assert_eq("a\nb\n".splitlines(True), ["a\n", "b\n"])

        # With keepends=False explicit
        assert_eq("a\nb".splitlines(False), ["a", "b"])

        # Empty string
        assert_eq("".splitlines(), [])
        assert_eq("".splitlines(True), [])

        # Mixed newlines (splitlines usually handles \n, \r\n, \r)
        # Note: Current implementation uses s.split_inclusive('\n') or s.lines()
        # s.lines() handles \n and \r\n.
        assert_eq("a\r\nb".splitlines(), ["a", "b"])
    "#,
    );
}

#[test]
fn test_title_extended() {
    assert::pass(
        r#"
        assert_eq("hello world".title(), "Hello World")
        assert_eq("HELLO WORLD".title(), "Hello World")
        assert_eq("HeLLo WoRLd".title(), "Hello World")

        # Non-alphabetic separators
        assert_eq("hello-world".title(), "Hello-World")
        assert_eq("hello'world".title(), "Hello'World")
        assert_eq("1st place".title(), "1St Place") # Python's title() behavior: 1St (since '1' is not alphabetic, 's' is start of new word)

        # Empty
        assert_eq("".title(), "")
    "#,
    );
}

#[test]
fn test_strip_variants_extended() {
    assert::pass(
        r#"
        # lstrip/rstrip/strip with args
        assert_eq("www.example.com".lstrip("cmwz."), "example.com")
        assert_eq("www.example.com".rstrip("omz."), "www.example.c")
        assert_eq("www.example.com".strip("w.moc"), "example")

        # Order of chars in arg shouldn't matter
        assert_eq("abc".strip("cba"), "")

        # Empty args
        assert_eq("  abc  ".strip(""), "  abc  ") # Strips nothing if chars is empty

        # Empty string
        assert_eq("".strip(), "")
        assert_eq("".lstrip("a"), "")
    "#,
    );
}

#[test]
fn test_remove_prefix_suffix_extended() {
    assert::pass(
        r#"
        s = "TestString"
        assert_eq(s.removeprefix("Test"), "String")
        assert_eq(s.removeprefix("NonExistent"), "TestString")
        assert_eq(s.removeprefix(""), "TestString")
        assert_eq(s.removeprefix(s), "")

        assert_eq(s.removesuffix("String"), "Test")
        assert_eq(s.removesuffix("NonExistent"), "TestString")
        assert_eq(s.removesuffix(""), "TestString")
        assert_eq(s.removesuffix(s), "")
    "#,
    );
}

#[test]
fn test_format_extended() {
    assert::pass(
        r#"
        # Basic
        assert_eq("{} {}".format("a", "b"), "a b")

        # Extra args ignored (Python does this, verify impl)
        # Impl: uses arg_idx, ignores extra
        assert_eq("{}".format("a", "b"), "a")

        # Not enough args -> IndexError handled below

        # No placeholders
        assert_eq("no placeholders".format("a"), "no placeholders")

        # Unclosed braces - verify behavior (impl dumps chars)
        assert_eq("{".format("a"), "{")
        assert_eq("}".format("a"), "}")
    "#,
    );

    assert::fail("'{}'.format()", "IndexError");
}

#[test]
fn test_partition_extended() {
    assert::pass(
        r#"
        # Sep found
        assert_eq("a.b".partition("."), ("a", ".", "b"))
        assert_eq("a.b".rpartition("."), ("a", ".", "b"))

        # Sep at start
        assert_eq(".b".partition("."), ("", ".", "b"))
        assert_eq(".b".rpartition("."), ("", ".", "b"))

        # Sep at end
        assert_eq("a.".partition("."), ("a", ".", ""))
        assert_eq("a.".rpartition("."), ("a", ".", ""))

        # Sep not found
        assert_eq("abc".partition("."), ("abc", "", ""))
        assert_eq("abc".rpartition("."), ("", "", "abc"))

        # Empty string
        assert_eq("".partition("."), ("", "", ""))

        # Multiple separators
        assert_eq("a.b.c".partition("."), ("a", ".", "b.c"))
        assert_eq("a.b.c".rpartition("."), ("a.b", ".", "c"))
    "#,
    );
}

#[test]
fn test_unicode_extended() {
    assert::pass(
        r#"
        s = "🔥"
        assert_eq(len(s), 4) # len() returns byte length in Eldritch

        assert_eq(s.codepoints(), [128293])
        assert_eq(s.elems(), ["🔥"])

        s2 = "a🔥b"
        assert_eq(s2.elems(), ["a", "🔥", "b"])

        # Reverse string with unicode
        assert_eq(s2[::-1], "b🔥a")
    "#,
    );
}
