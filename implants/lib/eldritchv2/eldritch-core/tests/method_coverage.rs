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
    assert::fail("l=[1]; l.insert()", "insert() takes exactly two arguments");
    assert::fail("l=[1]; l.append()", "append() takes exactly one argument");
    assert::fail("l=[1]; l.extend()", "extend() takes exactly one argument");
    assert::fail("l=[1]; l.extend(1)", "extend() expects an iterable");
    assert::fail("l=[1]; l.remove()", "remove() takes exactly one argument");
    assert::fail("l=[1]; l.index()", "index() takes exactly one argument");
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
    assert::fail("d={}; d.get()", "get() takes 1 or 2 arguments");
    assert::fail("d={}; d.get('a', 'b', 'c')", "get() takes 1 or 2 arguments");
    assert::fail("d={}; d.update()", "update() takes exactly one argument");
    assert::fail("d={}; d.update(1)", "update() requires a dictionary");
}

#[test]
fn test_set_edge_cases() {
    // Errors
    assert::fail("s={1}; s.add()", "add() takes exactly one argument");
    assert::fail(
        "s={1}; s.contains()",
        "contains() takes exactly one argument",
    );
    assert::fail(
        "s={1}; s.difference()",
        "difference() takes exactly one argument",
    );
    assert::fail("s={1}; s.difference(1)", "is not iterable");
    assert::fail("s={1}; s.discard()", "discard() takes exactly one argument");
    assert::fail(
        "s={1}; s.intersection()",
        "intersection() takes exactly one argument",
    );
    assert::fail(
        "s={1}; s.isdisjoint()",
        "isdisjoint() takes exactly one argument",
    );
    assert::fail(
        "s={1}; s.issubset()",
        "issubset() takes exactly one argument",
    );
    assert::fail(
        "s={1}; s.issuperset()",
        "issuperset() takes exactly one argument",
    );
    assert::fail("s={1}; s.remove()", "remove() takes exactly one argument");
    assert::fail("s={1}; s.remove(99)", "KeyError");
    assert::fail(
        "s={1}; s.symmetric_difference()",
        "symmetric_difference() takes exactly one argument",
    );
    assert::fail("s={1}; s.union()", "union() takes exactly one argument");
    assert::fail("s={1}; s.update()", "update() takes exactly one argument");
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
    assert::fail("''.startswith()", "startswith() takes 1 argument");
    assert::fail("''.endswith()", "endswith() takes 1 argument");
    assert::fail("''.removeprefix()", "removeprefix() takes 1 argument");
    assert::fail("''.removesuffix()", "removesuffix() takes 1 argument");
    assert::fail("''.find()", "find() takes 1 argument");
    assert::fail("''.index()", "index() takes 1 argument");
    assert::fail("''.index('z')", "ValueError: substring not found");
    assert::fail("''.rfind()", "rfind() takes 1 argument");
    assert::fail("''.rindex()", "rindex() takes 1 argument");
    assert::fail("''.rindex('z')", "ValueError: substring not found");
    assert::fail("''.count()", "count() takes 1 argument");
    assert::fail("''.replace('a')", "replace() takes 2 arguments");
    assert::fail("''.join()", "join() takes 1 argument");
    assert::fail("''.join(1)", "join() expects a list");
    assert::fail("''.join([1])", "join() expects list of strings");
    assert::fail("''.partition()", "partition() takes 1 argument");
    assert::fail("''.rpartition()", "rpartition() takes 1 argument");
    assert::fail("'{}'.format()", "tuple index out of range");

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
