mod assert;

#[test]
fn test_list_edge_cases() {
    // Insert with negative index
    assert::pass(
        r#"
        l = [1, 2, 3]
        l.insert(-1, 99)
        # insert at -1 (len + -1) = index 2. So [1, 2, 99, 3]
        assert_eq(l, [1, 2, 99, 3])

        l = [1]
        l.insert(-100, 99) # Should clamp to 0
        assert_eq(l, [99, 1])
    "#,
    );

    // Errors
    assert::fail(
        "l=[1]; l.insert('a', 1)",
        "insert() index must be an integer",
    );
    assert::fail("l=[1]; l.insert()", "TypeError: insert() takes exactly 2 arguments");
    assert::fail("l=[1]; l.append()", "TypeError: append() takes exactly 1 argument");
    assert::fail("l=[1]; l.extend()", "TypeError: extend() takes exactly 1 argument");
    assert::fail("l=[1]; l.extend(1)", "TypeError: extend() expects an iterable");
    assert::fail("l=[1]; l.remove()", "TypeError: remove() takes exactly 1 argument");
    assert::fail("l=[1]; l.index()", "TypeError: index() takes exactly 1 argument");
}

#[test]
fn test_dict_edge_cases() {
    // Get with default
    assert::pass(
        r#"
        d = {"a": 1}
        assert_eq(d.get("b", 2), 2)
        assert_eq(d.get("a", 99), 1)
        assert_eq(d.get("a"), 1)
    "#,
    );

    // Errors
    assert::fail("d={}; d.get()", "TypeError: get() takes between 1 and 2 arguments");
    assert::fail("d={}; d.get('a', 'b', 'c')", "TypeError: get() takes between 1 and 2 arguments");
    assert::fail("d={}; d.update()", "TypeError: update() takes exactly 1 argument");
    assert::fail("d={}; d.update(1)", "TypeError: update() requires a dictionary");
}

#[test]
fn test_set_edge_cases() {
    // Errors
    assert::fail("s={1}; s.add()", "TypeError: add() takes exactly 1 argument");
    assert::fail(
        "s={1}; s.contains()",
        "TypeError: contains() takes exactly 1 argument",
    );
    assert::fail(
        "s={1}; s.difference()",
        "TypeError: difference() takes exactly 1 argument",
    );
    assert::fail("s={1}; s.difference(1)", "TypeError: 'int' object is not iterable");
    assert::fail("s={1}; s.discard()", "TypeError: discard() takes exactly 1 argument");
    assert::fail(
        "s={1}; s.intersection()",
        "TypeError: intersection() takes exactly 1 argument",
    );
    assert::fail(
        "s={1}; s.isdisjoint()",
        "TypeError: isdisjoint() takes exactly 1 argument",
    );
    assert::fail(
        "s={1}; s.issubset()",
        "TypeError: issubset() takes exactly 1 argument",
    );
    assert::fail(
        "s={1}; s.issuperset()",
        "TypeError: issuperset() takes exactly 1 argument",
    );
    assert::fail("s={1}; s.remove()", "TypeError: remove() takes exactly 1 argument");
    assert::fail("s={1}; s.remove(99)", "KeyError");
    assert::fail(
        "s={1}; s.symmetric_difference()",
        "TypeError: symmetric_difference() takes exactly 1 argument",
    );
    assert::fail("s={1}; s.union()", "TypeError: union() takes exactly 1 argument");
    assert::fail("s={1}; s.update()", "TypeError: update() takes exactly 1 argument");
}

#[test]
fn test_string_edge_cases() {
    // Split/Splitlines
    assert::pass(
        r#"
        assert_eq("a b".split(), ["a", "b"])
        assert_eq("a,b".split(","), ["a", "b"])
        # splitlines with arg
        assert_eq("a\nb".splitlines(True), ["a\n", "b"])
        assert_eq("a\nb".splitlines(False), ["a", "b"])
    "#,
    );

    // Errors
    assert::fail("''.startswith()", "TypeError: startswith() takes exactly 1 argument");
    assert::fail("''.endswith()", "TypeError: endswith() takes exactly 1 argument");
    assert::fail("''.removeprefix()", "TypeError: removeprefix() takes exactly 1 argument");
    assert::fail("''.removesuffix()", "TypeError: removesuffix() takes exactly 1 argument");
    assert::fail("''.find()", "TypeError: find() takes exactly 1 argument");
    assert::fail("''.index()", "TypeError: index() takes exactly 1 argument");
    assert::fail("''.index('z')", "ValueError: substring not found");
    assert::fail("''.rfind()", "TypeError: rfind() takes exactly 1 argument");
    assert::fail("''.rindex()", "TypeError: rindex() takes exactly 1 argument");
    assert::fail("''.rindex('z')", "ValueError: substring not found");
    assert::fail("''.count()", "TypeError: count() takes exactly 1 argument");
    assert::fail("''.replace('a')", "TypeError: replace() takes exactly 2 arguments");
    assert::fail("''.join()", "TypeError: join() takes exactly 1 argument");
    assert::fail("''.join(1)", "TypeError: join() expects a list");
    assert::fail("''.join([1])", "TypeError: join() expects list of strings");
    assert::fail("''.partition()", "TypeError: partition() takes exactly 1 argument");
    assert::fail("''.rpartition()", "TypeError: rpartition() takes exactly 1 argument");
    assert::fail("'{}'.format()", "IndexError: tuple index out of range");

    // Coverage for format logic
    assert::pass(
        r#"
        assert_eq("{}{}".format("a", "b"), "ab")
        assert_eq("a{b}c".format(), "a{b}c") # No {} to replace
        assert_eq("{}".format(1), "1")
    "#,
    );

    // Coverage for partition fail cases
    assert::pass(
        r#"
        assert_eq("abc".partition("z"), ("abc", "", ""))
        assert_eq("abc".rpartition("z"), ("", "", "abc"))
    "#,
    );

    // Coverage for istitle mixed cases
    assert::pass(
        r#"
        assert_eq("Hello World".istitle(), True)
        assert_eq("Hello world".istitle(), False)
        assert_eq("hello World".istitle(), False)
        assert_eq("HELLO".istitle(), False)
        assert_eq("123".istitle(), False)
        assert_eq("".istitle(), False)
    "#,
    );

    // Coverage for islower/upper edge cases
    assert::pass(
        r#"
        assert_eq("123".islower(), False)
        assert_eq("123".isupper(), False)
        assert_eq("".islower(), False)
        assert_eq("".isupper(), False)
        assert_eq("a1".islower(), True)
        assert_eq("A1".isupper(), True)
    "#,
    );

    // Count empty string
    assert::pass(
        r#"
        assert_eq("abc".count(""), 4) # 3 chars + 1
    "#,
    );
}
