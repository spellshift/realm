mod assert;

#[test]
fn test_join_edge_cases() {
    assert::pass(
        r#"
        # Join empty list
        assert_eq("-".join([]), "")

        # Join list with one element
        assert_eq("-".join(["a"]), "a")

        # Join empty strings
        assert_eq("-".join(["", ""]), "-")

        # Join on empty string
        assert_eq("".join(["a", "b"]), "ab")
    "#,
    );
}

#[test]
fn test_join_errors() {
    assert::fail(
        r#"
        "-".join([1, 2])
    "#,
        "TypeError: join() expects list of strings",
    );

    assert::fail(
        r#"
        "-".join("not a list")
    "#,
        "TypeError: join() expects a list",
    );
}

#[test]
fn test_replace_edge_cases() {
    assert::pass(
        r#"
        s = "banana"
        assert_eq(s.replace("a", "o"), "bonono")
        assert_eq(s.replace("ana", "ono"), "bonona")

        # Replace empty string (inserts between every char)
        # Python: 'abc'.replace('', '-') -> '-a-b-c-'
        # Rust: "abc".replace("", "-") -> "-a-b-c-"
        assert_eq("abc".replace("", "-"), "-a-b-c-")

        # Replace non-existent
        assert_eq("abc".replace("d", "e"), "abc")
    "#,
    );
}

#[test]
fn test_replace_errors() {
    assert::fail(
        r#"
        "a".replace("b")
    "#,
        "replace() takes exactly 2 arguments",
    );
}

#[test]
fn test_find_index_errors() {
    // Currently implementation restricts to 1 argument
    assert::fail(
        r#"
        "abc".find("b", 1)
    "#,
        "find() takes exactly 1 argument",
    );

    assert::fail(
        r#"
        "abc".index("d")
    "#,
        "ValueError: substring not found",
    );

    assert::fail(
        r#"
        "abc".rindex("d")
    "#,
        "ValueError: substring not found",
    );
}

#[test]
fn test_split_edge_cases() {
    assert::pass(
        r#"
        # Split with no args (whitespace)
        assert_eq(" a  b c ".split(), ["a", "b", "c"])

        # Split with delimiter
        assert_eq("a,b,c".split(","), ["a", "b", "c"])

        # Split with empty parts
        assert_eq("a,,c".split(","), ["a", "", "c"])

        # Split empty string
        assert_eq("".split(","), [""])
        # "".split() -> [] (whitespace split on empty string is empty list)
        assert_eq("".split(), [])
    "#,
    );
}

#[test]
fn test_is_methods_unicode() {
    assert::pass(
        r#"
        # Unicode checks
        assert_eq("é".isalpha(), True)
        assert_eq("1".isalpha(), False)
        assert_eq(" ".isalpha(), False)

        assert_eq("abc".islower(), True)
        assert_eq("ABC".isupper(), True)

        # Mixed
        assert_eq("aB".islower(), False)
        assert_eq("aB".isupper(), False)
    "#,
    );
}

#[test]
fn test_format_errors() {
    assert::fail(
        r#"
        "{}".format()
    "#,
        "IndexError: tuple index out of range",
    );
}
