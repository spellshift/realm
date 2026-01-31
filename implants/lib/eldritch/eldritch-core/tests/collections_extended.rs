mod assert;

#[test]
fn test_list_extended_methods() {
    assert::pass(
        r#"
    l = [1, 2, 3]
    # Append
    l.append(4)
    assert_eq(l, [1, 2, 3, 4])

    # Extend with list
    l.extend([5, 6])
    assert_eq(l, [1, 2, 3, 4, 5, 6])

    # Extend with tuple
    l.extend((7, 8))
    assert_eq(l, [1, 2, 3, 4, 5, 6, 7, 8])

    # Insert
    l.insert(0, 0)
    assert_eq(l, [0, 1, 2, 3, 4, 5, 6, 7, 8])
    l.insert(len(l), 9)
    assert_eq(l[-1], 9)

    # Insert negative index
    # len is 10. insert(-1) -> index 9. Element at 9 is '9'.
    # Insert inserts *before* the element at that index.
    l.insert(-1, 8.5)
    assert_eq(l[-2], 8.5)
    assert_eq(l[-1], 9)

    # Remove
    l.remove(8.5)
    assert_eq(len(l), 10)
    l.remove(0)
    assert_eq(l[0], 1)

    # Index
    assert_eq(l.index(5), 4) # 1, 2, 3, 4, 5 -> index 4 (0-based)

    # Pop
    val = l.pop()
    assert_eq(val, 9)
    assert_eq(len(l), 8)

    # Sort
    l = [3, 1, 2]
    l.sort()
    assert_eq(l, [1, 2, 3])
    "#,
    );

    // Fail cases
    assert::fail("l=[1]; l.remove(2)", "ValueError");
    assert::fail("l=[1]; l.index(2)", "ValueError");
    assert::fail("l=[]; l.pop()", "pop from empty list");
    // Missing arguments
    assert::fail("l=[1]; l.append()", "append() takes exactly 1 argument");
}

#[test]
fn test_list_operations_builtins() {
    assert::pass(
        r#"
    l1 = [1, 2]
    l2 = [3, 4]

    # Addition
    l3 = l1 + l2
    assert_eq(l3, [1, 2, 3, 4])
    assert_eq(l1, [1, 2])

    # In / Not In
    assert(1 in l1)
    assert(3 not in l1)

    # Len
    assert_eq(len(l1), 2)

    # Min/Max
    l = [1, 2, 3]
    assert_eq(min(l), 1)
    assert_eq(max(l), 3)

    # Any/All
    assert(any([False, True]))
    assert(not all([False, True]))
    assert(all([True, True]))

    # Sorted
    l = [3, 1, 2]
    l2 = sorted(l)
    assert_eq(l2, [1, 2, 3])
    assert_eq(l, [3, 1, 2]) # Original unchanged

    # Reversed
    l = [1, 2]
    # reversed returns list in Eldritch (based on sorted returning list, safer to assume iterable/list)
    r = reversed(l)
    assert_eq(list(r), [2, 1])
    "#,
    );
}

#[test]
fn test_dict_extended() {
    assert::pass(
        r#"
    d = {"a": 1, "b": 2}

    # Keys/Values/Items
    k = d.keys()
    assert("a" in k)
    v = d.values()
    assert(1 in v)
    i = d.items()
    assert(len(i) == 2)

    # Get
    assert_eq(d.get("a"), 1)
    assert_eq(d.get("z"), None)
    assert_eq(d.get("z", 3), 3)

    # Update
    d.update({"c": 3})
    assert_eq(d["c"], 3)

    # Popitem
    # Order is implementation dependent (BTreeMap is sorted by key)
    # Keys: a, b, c. Last is c.
    item = d.popitem()
    assert_eq(item[0], "c")
    assert_eq(item[1], 3)
    assert_eq(len(d), 2)

    # Addition (Merge)
    d1 = {"a": 1}
    d2 = {"b": 2}
    d3 = d1 + d2
    assert_eq(d3["a"], 1)
    assert_eq(d3["b"], 2)
    "#,
    );

    // Fail
    assert::fail("d={}; d.popitem()", "empty");
}

#[test]
fn test_set_extended() {
    assert::pass(
        r#"
    s = {1, 2}

    # Add
    s.add(3)
    assert(3 in s)

    # Discard vs Remove
    s.discard(99) # No error

    # Operations
    s1 = {1, 2}
    s2 = {2, 3}

    # Union
    u = s1.union(s2)
    assert_eq(len(u), 3)

    # Intersection
    i = s1.intersection(s2)
    assert_eq(len(i), 1)
    assert(2 in i)

    # Difference
    d = s1.difference(s2)
    assert_eq(len(d), 1)
    assert(1 in d)

    # Sym Diff
    sd = s1.symmetric_difference(s2)
    assert_eq(len(sd), 2) # 1, 3
    assert(1 in sd)
    assert(3 in sd)

    # Is...
    assert(s1.issubset({1, 2, 3}))
    assert({1, 2, 3}.issuperset(s1))

    # Disjoint
    assert(s1.isdisjoint({3, 4})) # s1={1,2}. True.
    assert(not s1.isdisjoint({2, 3}))

    # Addition (Union)
    s3 = s1 + s2
    assert_eq(len(s3), 3)

    # Update (in place)
    s1.update(s2)
    assert_eq(len(s1), 3)
    "#,
    );

    assert::fail("s={1}; s.remove(99)", "KeyError");
}

#[test]
fn test_interplay() {
    assert::pass(
        r#"
    # List <-> Set
    l = [1, 2, 2, 3]
    s = set(l)
    assert_eq(len(s), 3)
    l2 = list(s)
    assert_eq(len(l2), 3)

    # List <-> Tuple
    t = tuple([1, 2])
    assert_eq(t[0], 1)
    l = list(t)
    assert_eq(l[0], 1)

    # Dict keys from tuple (Not supported natively as map keys must be strings currently in Eldritch?)
    # Wait, methods.rs: "Dict keys must be strings".
    # eval.rs: "Dictionary keys must be strings."
    # So {(1,2): 1} should FAIL.
    # Let's verify that.

    # Zip
    z = zip([1, 2], [3, 4])
    # zip returns list of tuples in Eldritch
    assert_eq(z[0], (1, 3))
    assert_eq(z[1], (2, 4))

    # Dict from zip
    d = dict(zip(["a", "b"], [1, 2]))
    assert_eq(d["a"], 1)
    assert_eq(d["b"], 2)

    # Nested
    l = [[1, 2], [3, 4]]
    assert_eq(l[0][1], 2)

    d = {"inner": {"a": 1}}
    assert_eq(d["inner"]["a"], 1)
    "#,
    );
}

#[test]
fn test_tuple_methods() {
    // Tuples currently have no methods in Eldritch (unlike Python which has index, count)
    assert::fail("t=(1,); t.index(1)", "has no method 'index'");
    assert::fail("t=(1,); t.count(1)", "has no method 'count'");
}
