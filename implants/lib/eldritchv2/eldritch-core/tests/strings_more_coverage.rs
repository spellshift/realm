mod assert;

#[test]
fn test_replace_edge_cases() {
    assert::pass(
        r#"
        # Basic replace
        assert_eq("aba".replace("a", "o"), "obo")

        # Replace resulting in empty string
        assert_eq("aaa".replace("a", ""), "")

        # Replace empty string (Rust behavior: inserts at every position?)
        # Let's verify what happens.
        # "abc".replace("", "-") -> "-a-b-c-" in Rust?
        # Actually usually it's not allowed or specific behavior.
        # Python: "abc".replace("", "-") -> "-a-b-c-"
        # Rust std::str::replace: "" matches at start and end?
        # Let's test it.
        # assert_eq("abc".replace("", "-"), "-a-b-c-")

        # Replace not found
        assert_eq("abc".replace("d", "x"), "abc")

        # Multiple occurrences
        assert_eq("banana".replace("a", "o"), "bonono")

        # Overlapping - Rust replace is non-overlapping
        assert_eq("aaaa".replace("aa", "b"), "bb")
    "#,
    );
}

#[test]
fn test_find_index_arg_validation() {
    // These methods strictly require 1 argument.
    assert::fail(
        "'abc'.find('b', 1)",
        "TypeError: find() takes exactly 1 argument",
    );
    assert::fail(
        "'abc'.index('b', 1)",
        "TypeError: index() takes exactly 1 argument",
    );
    assert::fail(
        "'abc'.rfind('b', 1)",
        "TypeError: rfind() takes exactly 1 argument",
    );
    assert::fail(
        "'abc'.rindex('b', 1)",
        "TypeError: rindex() takes exactly 1 argument",
    );
    assert::fail(
        "'abc'.count('b', 1)",
        "TypeError: count() takes exactly 1 argument",
    );

    // Missing argument
    assert::fail("'abc'.find()", "TypeError: find() takes exactly 1 argument");
}

#[test]
fn test_join_edge_cases() {
    assert::pass(
        r#"
        # Join empty list
        assert_eq("-".join([]), "")

        # Join list with one element
        assert_eq("-".join(["a"]), "a")

        # Join empty strings
        assert_eq("-".join(["", "", ""]), "--")
    "#,
    );

    // Join with non-string
    assert::fail(
        "'-'.join(['a', 1])",
        "TypeError: join() expects list of strings",
    );

    // Join with non-list
    assert::fail("'-'.join('abc')", "TypeError: join() expects a list");
}

#[test]
fn test_count_edge_cases() {
    assert::pass(
        r#"
        assert_eq("aaa".count("a"), 3)
        assert_eq("aaa".count("aa"), 1) # Non-overlapping
        assert_eq("abc".count("d"), 0)

        # Empty string count
        # Rust matches().count() with empty string might behave interestingly or infinite loop if not handled?
        # The implementation in str.rs says:
        # if sub.is_empty() { return Ok(Value::Int((s.len() + 1) as i64)); }
        assert_eq("abc".count(""), 4)
        assert_eq("".count(""), 1)
    "#,
    );
}

#[test]
fn test_is_methods_edge_cases() {
    assert::pass(
        r#"
        # istitle
        assert("Hello World".istitle())
        assert(not "Hello world".istitle())
        assert(not "hello World".istitle())
        assert("1St Place".istitle()) # Based on Rust impl behavior check in strings_extended_coverage
        assert(not "".istitle())
        assert("A".istitle())
        assert(not "a".istitle())

        # islower / isupper with non-cased chars
        assert("abc1".islower()) # 1 is ignored, abc is lower
        assert("ABC1".isupper())
        assert(not "123".islower()) # No cased characters -> false (impl requires at least one cased char?)
        # Impl: !s.is_empty() && s.chars().any(|c| c.is_alphabetic()) && s == s.to_lowercase()
        assert(not "123".islower())
        assert(not "123".isupper())
        assert(not "".islower())
    "#,
    );
}
