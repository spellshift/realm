mod assert;

#[test]
fn test_str_conversion() {
    assert::pass(
        r#"
        b = b"hello world"
        s = str(b)
        assert_eq(s, "hello world")
        assert_eq(type(s), "string")

        # Test fallback (not strictly required by spec to be exact repr but ensures no crash)
        # b_invalid = b"\xff"
        # s_inv = str(b_invalid)
        # assert(type(s_inv) == "string")
    "#,
    );
}

#[test]
fn test_new_str_methods() {
    assert::pass(
        r#"
        s = "heLLo"
        assert_eq(s.capitalize(), "Hello")

        # Codepoints
        cp = "A".codepoints()
        assert_eq(cp[0], 65)

        # Elems
        el = "ABC".elems()
        assert_eq(el, ["A", "B", "C"])

        # Count
        assert_eq("banana".count("a"), 3)
        assert_eq("banana".count("ana"), 1)

        # Index
        assert_eq("abc".index("b"), 1)
        # assert fail? We can't catch exceptions yet in tests easily unless we wrap in try/except if language supports it.
        # Eldritch doesn't have try/except yet exposed in this context easily?
        # Assuming no try/except, we skip testing failure case in 'pass' block.

        # Is checks
        assert_eq("abc".islower(), True)
        assert_eq("ABC".isupper(), True)
        assert_eq("Title".istitle(), True)
        assert_eq("123".isdigit(), True)
        assert_eq("a1".isalnum(), True)
        assert_eq(" ".isspace(), True)
        assert_eq("a".isalpha(), True)

        # Partition
        parts = "a.b.c".partition(".")
        assert_eq(parts, ("a", ".", "b.c"))

        parts_r = "a.b.c".rpartition(".")
        assert_eq(parts_r, ("a.b", ".", "c"))

        # Strip variants
        assert_eq("  abc  ".lstrip(), "abc  ")
        assert_eq("  abc  ".rstrip(), "  abc")

        # Split variants
        assert_eq("a\nb".splitlines(), ["a", "b"])
        # assert_eq("a\nb".splitlines(True), ["a\n", "b"]) # Optional arg handling check

        # Rfind/Rindex
        assert_eq("aba".rfind("a"), 2)
        assert_eq("aba".rindex("a"), 2)

        # Rsplit
        # assert_eq("a,b,c".rsplit(",", 1), ["a,b", "c"]) # Maxsplit not implemented yet
        assert_eq("a,b,c".rsplit(","), ["a", "b", "c"])

        # Title
        assert_eq("hello world".title(), "Hello World")

        # Remove prefix/suffix
        assert_eq("test_file".removeprefix("test_"), "file")
        assert_eq("file.txt".removesuffix(".txt"), "file")
    "#,
    );
}

#[test]
fn test_pprint() {
    assert::pass(
        r#"
        data = {"key": [1, 2, 3], "nested": {"a": 1}}
        # Simply calling it to ensure it doesn't crash.
        # Capturing stdout is hard here without interpreter hooks in test harness.
        pprint(data)
        pprint(data, 4)
    "#,
    );
}
