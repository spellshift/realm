mod assert;

#[test]
fn test_set_method_arguments() {
    // add
    assert::fail("s={1}; s.add()", "add() takes exactly 1 argument");
    assert::fail("s={1}; s.add(1, 2)", "add() takes exactly 1 argument");

    // clear
    assert::fail("s={1}; s.clear(1)", "clear() takes exactly 0 arguments");

    // contains (internal method really, but exposed via 'in')
    // assert::fail("s={1}; s.contains()", "contains() takes exactly 1 argument"); // Not directly callable usually

    // difference
    assert::fail(
        "s={1}; s.difference()",
        "difference() takes exactly 1 argument",
    );

    // discard
    assert::fail("s={1}; s.discard()", "discard() takes exactly 1 argument");

    // intersection
    assert::fail(
        "s={1}; s.intersection()",
        "intersection() takes exactly 1 argument",
    );

    // isdisjoint
    assert::fail(
        "s={1}; s.isdisjoint()",
        "isdisjoint() takes exactly 1 argument",
    );

    // issubset
    assert::fail("s={1}; s.issubset()", "issubset() takes exactly 1 argument");

    // issuperset
    assert::fail(
        "s={1}; s.issuperset()",
        "issuperset() takes exactly 1 argument",
    );

    // pop
    assert::fail("s={1}; s.pop(1)", "pop() takes exactly 0 arguments");

    // remove
    assert::fail("s={1}; s.remove()", "remove() takes exactly 1 argument");

    // symmetric_difference
    assert::fail(
        "s={1}; s.symmetric_difference()",
        "symmetric_difference() takes exactly 1 argument",
    );

    // union
    assert::fail("s={1}; s.union()", "union() takes exactly 1 argument");

    // update
    assert::fail("s={1}; s.update()", "update() takes exactly 1 argument");
}

#[test]
fn test_set_conversion_types() {
    // List
    assert::pass(
        r#"
        s = {1}
        s.update([2, 3])
        assert_eq(s, {1, 2, 3})
    "#,
    );

    // Tuple
    assert::pass(
        r#"
        s = {1}
        s.update((2, 3))
        assert_eq(s, {1, 2, 3})
    "#,
    );

    // String (chars)
    assert::pass(
        r#"
        s = {"a"}
        s.update("bc")
        assert_eq(s, {"a", "b", "c"})
    "#,
    );

    // Dict (keys)
    assert::pass(
        r#"
        s = {1}
        s.update({2: "a", 3: "b"})
        assert_eq(s, {1, 2, 3})
    "#,
    );

    // Error: Non-iterable
    assert::fail(
        "s={1}; s.update(1)",
        "TypeError: 'int' object is not iterable",
    );
}

#[test]
fn test_set_empty_operations() {
    assert::pass(
        r#"
        s1 = set()
        s2 = {1, 2}

        # Union with empty
        assert_eq(s1.union(s2), {1, 2})
        assert_eq(s2.union(s1), {1, 2})

        # Intersection with empty
        assert_eq(s1.intersection(s2), set())
        assert_eq(s2.intersection(s1), set())

        # Difference with empty
        assert_eq(s1.difference(s2), set())
        assert_eq(s2.difference(s1), {1, 2})

        # Symmetric difference with empty
        assert_eq(s1.symmetric_difference(s2), {1, 2})
        assert_eq(s2.symmetric_difference(s1), {1, 2})
    "#,
    );
}

#[test]
fn test_set_pop_remove_errors() {
    // Pop empty
    assert::fail("s=set(); s.pop()", "KeyError: pop from empty set");

    // Remove missing
    assert::fail("s={1}; s.remove(2)", "KeyError: 2");

    // Discard missing (should not fail)
    assert::pass(
        r#"
        s = {1}
        s.discard(2)
        assert_eq(s, {1})
    "#,
    );
}

#[test]
fn test_set_comparisons_extended() {
    assert::pass(
        r#"
        s1 = {1, 2}
        s2 = {1, 2, 3}
        s3 = {4, 5}

        # issubset
        assert(s1.issubset(s2))
        assert(s1.issubset(s1))
        assert(not s2.issubset(s1))
        assert(set().issubset(s1)) # Empty set is subset of any set

        # issuperset
        assert(s2.issuperset(s1))
        assert(s1.issuperset(s1))
        assert(not s1.issuperset(s2))
        assert(s1.issuperset(set())) # Any set is superset of empty set

        # isdisjoint
        assert(s1.isdisjoint(s3))
        assert(not s1.isdisjoint(s2))
        assert(s1.isdisjoint(set()))
    "#,
    );
}
