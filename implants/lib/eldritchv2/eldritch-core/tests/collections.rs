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
fn test_sets() {
    assert::pass(
        r#"
        myset = {1, 2}
        assert_eq(len(myset), 2)
        assert(1 in myset)
        assert(2 in myset)

        myset.add(3)
        assert_eq(len(myset), 3)
        assert(3 in myset)
        myset.add(3) # Duplicate
        assert_eq(len(myset), 3)

        myset.clear()
        assert_eq(len(myset), 0)

        myset = {1, 2}
        assert(myset.contains(1))
        assert(not myset.contains(3))

        myset.discard(1)
        assert_eq(len(myset), 1)
        myset.discard(3) # Should not raise error
        assert_eq(len(myset), 1)
    "#,
    );
}

#[test]
fn test_set_operations() {
    assert::pass(
        r#"
        s1 = {1, 2, 3}
        s2 = {2, 3, 4}

        # Difference
        diff = s1.difference(s2)
        assert_eq(len(diff), 1)
        assert(1 in diff)

        # Intersection
        inter = s1.intersection(s2)
        assert_eq(len(inter), 2)
        assert(2 in inter)
        assert(3 in inter)

        # Is disjoint
        assert(not s1.isdisjoint(s2))
        assert({1}.isdisjoint({2}))

        # Subset/Superset
        assert({1}.issubset(s1))
        assert(s1.issuperset({1}))

        # Symmetric Difference
        sym = s1.symmetric_difference(s2)
        assert_eq(len(sym), 2)
        assert(1 in sym)
        assert(4 in sym)

        # Union
        u = s1.union(s2)
        assert_eq(len(u), 4)

        # Update
        s1.update(s2)
        assert_eq(len(s1), 4)
    "#,
    );
}

#[test]
fn test_set_iterable_args() {
    assert::pass(
        r#"
        s = {1}
        s.update([2])
        assert_eq(len(s), 2)

        assert_eq(len(s.union([3])), 3)
        assert_eq(len(s.intersection([1, 5])), 1)
        assert_eq(len(s.difference([2])), 1)
    "#,
    );
}

#[test]
fn test_set_pop_remove() {
    assert::pass(
        r#"
        myset = {1, 2}
        # Sets are ordered in Eldritch (BTreeSet), so pop (last) should return 2
        val = myset.pop()
        assert_eq(val, 2)
        assert_eq(len(myset), 1)
        assert(1 in myset)

        myset.remove(1)
        assert_eq(len(myset), 0)
    "#,
    );
    assert::fail("s = {1}; s.remove(2)", "KeyError");
    assert::fail("s = set(); s.pop()", "empty");
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

#[test]
fn test_nan_remove_issue_1293() {
    assert::pass(
        r#"
        # Reproduction of issue #1293 using float("nan")

        # This confirms we can create NaN
        nan_val = float("nan")

        # Case 1: Simple remove
        l = [nan_val]
        # This should remove the element if we fix the bug.
        # Currently it fails because NaN != NaN.
        l.remove(nan_val)

        assert_eq(len(l), 0)

        # Case 2: The loop scenario from the issue
        my_list = [1, 2, nan_val]

        for i in my_list:
             if i % 2 != 0:
                 my_list.remove(i)

        # 1 % 2 != 0 -> remove(1). List becomes [2, nan_val].
        # Next iter: 2. 2 % 2 == 0. Skip.
        # Next iter: nan_val. nan_val % 2 is nan_val. nan_val != 0 is True.
        # remove(nan_val) -> Should work.

        # Check final list content
        # 1 removed. 2 kept. nan_val removed.
        # Expected: [2]

        # Wait, does removing elements during iteration skip elements?
        # Initial: [1, 2, nan_val]
        # i=1. Remove 1. List=[2, nan_val].
        # Next index is 1. Element at index 1 is nan_val.
        # So 2 is skipped?
        # If 2 is skipped, it remains in the list.
        # i=nan_val. Remove nan_val. List=[2].

        assert_eq(my_list, [2])

        # Test index() consistency
        l2 = [nan_val]
        idx = l2.index(nan_val)
        assert_eq(idx, 0)
    "#,
    );
}

#[test]
fn test_nan_in_check() {
    assert::pass(
        r#"
        nan = float("nan")
        l = [nan]
        assert(nan in l)

        t = (nan,)
        assert(nan in t)

        nan2 = float("nan")
        assert(nan2 in l)
    "#,
    );
}
