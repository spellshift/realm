mod assert;

#[test]
fn test_lists() {
    assert::pass(
        r#"
        l = [10, 20, 30]
        assert_eq(l[0], 10)
        assert_eq(l[-1], 30)

        # Slicing
        assert_eq(l[::-1], [30, 20, 10])
        assert_eq(l[0:2], [10, 20])
    "#,
    );

    // Error cases
    assert::fail("l = [1]; l[5]", "List index out of range");
}

#[test]
fn test_list_methods() {
    assert::pass(
        r#"
        l = [1]
        l.append(2)
        assert_eq(l, [1, 2])

        l.extend([3, 4])
        assert_eq(l, [1, 2, 3, 4])

        l.insert(1, 5)
        assert_eq(l, [1, 5, 2, 3, 4])

        l.remove(5)
        assert_eq(l, [1, 2, 3, 4])

        assert_eq(l.index(3), 2)

        assert_eq(l.pop(), 4)
        assert_eq(l, [1, 2, 3])

        l = [3, 1, 2]
        l.sort()
        assert_eq(l, [1, 2, 3])
    "#,
    );

    assert::fail("l = [1]; l.remove(99)", "ValueError");
    assert::fail("l = [1]; l.index(99)", "ValueError");
    assert::fail("l = []; l.pop()", "pop from empty list");
    assert::fail("l = [1]; l.unknown()", "has no method");
}

#[test]
fn test_tuples() {
    assert::pass(
        r#"
        t = (1, 2, 3)
        assert_eq(t[0], 1)
        assert_eq(len(t), 3)
        assert_eq((1,), (1,))
        assert_eq((), ())

        # Slicing
        assert_eq(t[::-1], (3, 2, 1))
    "#,
    );

    assert::fail("t = (1,); t[5]", "Tuple index out of range");
}

#[test]
fn test_dictionaries() {
    assert::pass(
        r#"
        d = {"a": 1, "b": 2}
        assert_eq(len(d), 2)
        assert_eq(d["a"], 1)

        # Nested
        d2 = {"inner": [1, 2]}
        assert_eq(d2["inner"][0], 1)
    "#,
    );

    assert::fail("d = {}; d['missing']", "KeyError");
}

#[test]
fn test_dict_methods() {
    assert::pass(
        r#"
        d = {"a": 1, "b": 2}

        assert_eq(len(d.keys()), 2)
        assert_eq(len(d.values()), 2)
        assert_eq(len(d.items()), 2)

        assert_eq(d.get("a"), 1)
        assert_eq(d.get("c"), None)
        assert_eq(d.get("c", 10), 10)

        d.update({"c": 3})
        assert_eq(d["c"], 3)

        d.popitem()
        assert_eq(len(d), 2)
    "#,
    );

    assert::fail("d={}; d.popitem()", "empty");
}

#[test]
fn test_comprehensions() {
    assert::pass(
        r#"
        l = [x * 2 for x in [1, 2, 3]]
        assert_eq(l, [2, 4, 6])

        l = [x for x in [1, 2, 3, 4] if x > 2]
        assert_eq(l, [3, 4])

        # Scoping
        x = 100
        l = [x for x in [1, 2]]
        assert_eq(x, 100)
    "#,
    );

    assert::pass(
        r#"
        d = {str(x): x*x for x in [1, 2]}
        assert_eq(d["1"], 1)
        assert_eq(d["2"], 4)
    "#,
    );
}

#[test]
fn test_list_addition() {
    assert::pass(
        r#"
        l1 = [1, 2]
        l2 = [3, 4]
        l3 = l1 + l2
        assert_eq(l3, [1, 2, 3, 4])
        assert_eq(l1, [1, 2]) # Originals unchanged
        assert_eq(l2, [3, 4])

        l1 += [5]
        assert_eq(l1, [1, 2, 5])
    "#,
    );
}

#[test]
fn test_set_addition() {
    assert::pass(
        r#"
        s1 = {1, 2}
        s2 = {2, 3}
        s3 = s1 + s2
        assert_eq(s3, {1, 2, 3})

        # Order shouldn't matter for sets, but equality check handles it.
        # Ensure originals are unchanged
        assert_eq(s1, {1, 2})
        assert_eq(s2, {2, 3})

        s1 += {4}
        assert_eq(s1, {1, 2, 4})
    "#,
    );
}
