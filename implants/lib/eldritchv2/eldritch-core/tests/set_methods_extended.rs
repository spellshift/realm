mod assert;

#[test]
fn test_set_methods_with_different_iterables() {
    // union with list, tuple, string, dict
    assert::pass(
        r#"
        s = {1}
        assert_eq(s.union([2]), {1, 2})
        assert_eq(s.union((3,)), {1, 3})
        assert_eq(s.union("4"), {1, "4"})
        assert_eq(s.union({5: "a"}), {1, 5})
    "#,
    );

    // intersection
    assert::pass(
        r#"
        s = {1, 2, 3}
        assert_eq(s.intersection([2, 3, 4]), {2, 3})
        assert_eq(s.intersection((1, 4)), {1})
        s_char = {"a", "b"}
        assert_eq(s_char.intersection("ac"), {"a"})
        assert_eq(s.intersection({2: "v", 5: "x"}), {2})
    "#,
    );

    // difference
    assert::pass(
        r#"
        s = {1, 2, 3}
        assert_eq(s.difference([2, 4]), {1, 3})
        assert_eq(s.difference((1, 3)), {2})
        s_char = {"a", "b"}
        assert_eq(s_char.difference("a"), {"b"})
        assert_eq(s.difference({2: "x"}), {1, 3})
    "#,
    );

    // symmetric_difference
    assert::pass(
        r#"
        s = {1, 2}
        assert_eq(s.symmetric_difference([2, 3]), {1, 3})
        assert_eq(s.symmetric_difference((1, 3)), {2, 3})
        s_char = {"a", "b"}
        assert_eq(s_char.symmetric_difference("ac"), {"b", "c"})
        assert_eq(s.symmetric_difference({2: "x", 3: "y"}), {1, 3})
    "#,
    );
}

#[test]
fn test_set_comparisons_with_iterables() {
    // isdisjoint
    assert::pass(
        r#"
        s = {1, 2}
        assert(s.isdisjoint([3, 4]))
        assert(not s.isdisjoint([2, 3]))
        assert(s.isdisjoint((3, 4)))
        assert(s.isdisjoint("abc"))
        assert(s.isdisjoint({3: "a"}))
    "#,
    );

    // issubset
    assert::pass(
        r#"
        s = {1, 2}
        assert(s.issubset([1, 2, 3]))
        assert(not s.issubset([1]))
        assert(s.issubset((1, 2, 3)))
        s_char = {"a"}
        assert(s_char.issubset("ab"))
        assert(s.issubset({1: "a", 2: "b", 3: "c"}))
    "#,
    );

    // issuperset
    assert::pass(
        r#"
        s = {1, 2, 3}
        assert(s.issuperset([1, 2]))
        assert(not s.issuperset([1, 4]))
        assert(s.issuperset((1, 2)))
        s_char = {"a", "b"}
        assert(s_char.issuperset("a"))
        assert(s.issuperset({1: "a"}))
    "#,
    );
}

#[test]
fn test_set_mixed_types() {
    assert::pass(
        r#"
        s = {1, "a", 2.5}
        assert_eq(len(s), 3)
        assert(1 in s)
        assert("a" in s)
        assert(2.5 in s)
    "#,
    );

    // Check type separation
    assert::pass(
        r#"
        s = {1}
        s.add(True)
        assert_eq(len(s), 2) # {1, True}
        assert(1 in s)
        assert(True in s)
    "#,
    );

    // Floats and Ints equality
    assert::pass(
        r#"
        s = {1}
        assert(1.0 in s)
        s.add(1.0)
        # Should not add if equal
        assert_eq(len(s), 1)

        s = {1.0}
        assert(1 in s)
        s.add(1)
        assert_eq(len(s), 1)
    "#,
    );
}

#[test]
fn test_pop_ordering() {
    // Sets are BTreeSets, so ordered.
    // pop() returns the last element.
    assert::pass(
        r#"
        s = {3, 1, 2}
        # Sorted: 1, 2, 3
        assert_eq(s.pop(), 3)
        assert_eq(s.pop(), 2)
        assert_eq(s.pop(), 1)
        assert_eq(len(s), 0)
    "#,
    );

    // Mixed types ordering:
    // None(0) < Bool(1) < Int(2) < Float(3) < String(4)
    assert::pass(
        r#"
        s = {1, "a", True, None}
        # Expected order: None, True, 1, "a"

        assert_eq(s.pop(), "a")
        assert_eq(s.pop(), 1)
        assert_eq(s.pop(), True)
        assert_eq(s.pop(), None)
    "#,
    );
}

#[test]
fn test_recursive_structure_in_set() {
    // Attempt to put a tuple in a set
    assert::pass(
        r#"
        s = {(1, 2), (3, 4)}
        assert_eq(len(s), 2)
        assert((1, 2) in s)

        s.add((1, 2))
        assert_eq(len(s), 2)
    "#,
    );

    // Attempt to put a list in a set (Value::List implements Ord)
    assert::pass(
        r#"
        l = [1, 2]
        s = {l}
        assert(l in s)
        assert([1, 2] in s)

        # Mutation test - we just ensure it doesn't crash
        l.append(3)
        assert(len(s) == 1)
    "#,
    );
}
