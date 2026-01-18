mod assert;

#[test]
fn test_string_split_comprehensive() {
    assert::pass(r#"
        # Empty string
        assert_eq("".split(","), [""])
        # No args -> split by whitespace
        assert_eq("".split(), [])
        assert_eq("   ".split(), [])

        # Split by whitespace (default)
        assert_eq("a b c".split(), ["a", "b", "c"])
        assert_eq("a  b   c".split(), ["a", "b", "c"])
        assert_eq("  a b  ".split(), ["a", "b"])
        assert_eq("a\tb\nc".split(), ["a", "b", "c"])

        # Split by delimiter
        assert_eq("a,b,c".split(","), ["a", "b", "c"])
        assert_eq("a,,c".split(","), ["a", "", "c"])
        assert_eq(",a,b,".split(","), ["", "a", "b", ""])

        # Delimiter not found
        assert_eq("abc".split(","), ["abc"])

        # Extra arguments (maxsplit) are currently accepted but ignored
        assert_eq("a,b,c".split(",", 1), ["a", "b", "c"])
    "#);
}

#[test]
fn test_string_splitlines_comprehensive() {
    assert::pass(r#"
        assert_eq("a\nb\nc".splitlines(), ["a", "b", "c"])
        assert_eq("a\nb\nc\n".splitlines(), ["a", "b", "c"])
        assert_eq("".splitlines(), [])
        assert_eq("a".splitlines(), ["a"])

        # keepends=True
        assert_eq("a\nb".splitlines(True), ["a\n", "b"])
        assert_eq("a\nb\n".splitlines(True), ["a\n", "b\n"])

        # keepends=False (explicit)
        assert_eq("a\nb".splitlines(False), ["a", "b"])
    "#);
}

#[test]
fn test_string_replace_comprehensive() {
    assert::pass(r#"
        assert_eq("aba".replace("a", "o"), "obo")
        assert_eq("aaaa".replace("aa", "b"), "bb") # Non-overlapping
        assert_eq("hello".replace("l", ""), "heo")
        assert_eq("hello".replace("x", "y"), "hello")

        # Empty string cases
        assert_eq("".replace("a", "b"), "")
        # Replacing empty string inserts replacement between chars
        assert_eq("abc".replace("", "-"), "-a-b-c-")
    "#);
}

#[test]
fn test_string_strip_comprehensive() {
    assert::pass(r#"
        # No args -> whitespace
        assert_eq("  abc  ".strip(), "abc")
        assert_eq(" \t\n abc \r\n ".strip(), "abc")

        # With args
        assert_eq("...abc...".strip("."), "abc")
        assert_eq("xyxabcyxy".strip("xy"), "abc")
        assert_eq("abc".strip("z"), "abc")

        # lstrip / rstrip
        assert_eq("  abc  ".lstrip(), "abc  ")
        assert_eq("  abc  ".rstrip(), "  abc")
        assert_eq("...abc...".lstrip("."), "abc...")
        assert_eq("...abc...".rstrip("."), "...abc")
    "#);
}

#[test]
fn test_string_find_index_comprehensive() {
    assert::pass(r#"
        s = "hello world"
        assert_eq(s.find("o"), 4)
        assert_eq(s.rfind("o"), 7)
        assert_eq(s.find("x"), -1)
        assert_eq(s.rfind("x"), -1)

        assert_eq(s.index("o"), 4)
        assert_eq(s.rindex("o"), 7)

        # Empty substring
        assert_eq(s.find(""), 0)
        assert_eq(s.rfind(""), 11) # Length of "hello world" is 11
    "#);

    assert::fail(r#" "abc".index("z") "#, "ValueError: substring not found");
    assert::fail(r#" "abc".rindex("z") "#, "ValueError: substring not found");
}

#[test]
fn test_string_count_comprehensive() {
    assert::pass(r#"
        assert_eq("aaaa".count("aa"), 2) # Non-overlapping
        assert_eq("abc".count("b"), 1)
        assert_eq("abc".count("z"), 0)

        # Empty substring -> len + 1
        assert_eq("abc".count(""), 4)
    "#);
}

#[test]
fn test_string_partition_comprehensive() {
    assert::pass(r#"
        assert_eq("a.b.c".partition("."), ("a", ".", "b.c"))
        assert_eq("abc".partition("."), ("abc", "", ""))
        assert_eq("".partition("."), ("", "", ""))

        assert_eq("a.b.c".rpartition("."), ("a.b", ".", "c"))
        assert_eq("abc".rpartition("."), ("", "", "abc"))
        assert_eq("".rpartition("."), ("", "", ""))
    "#);
}

#[test]
fn test_string_format_basic() {
    assert::pass(r#"
        assert_eq("Hello {}".format("World"), "Hello World")
        assert_eq("{} {}".format("A", "B"), "A B")
        assert_eq("Value: {}".format(123), "Value: 123")

        # Extra args ignored
        assert_eq("{}".format("A", "B"), "A")
    "#);

    assert::fail(r#" "{} {}".format("A") "#, "IndexError");
}
