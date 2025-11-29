mod assert;

#[test]
fn test_map_filter_reduce() {
    // Map
    assert::pass(
        r#"
        l = [1, 2, 3]
        res = map(lambda x: x * 2, l)
        assert_eq(res, [2, 4, 6])
    "#,
    );

    // Filter
    assert::pass(
        r#"
        l = [1, 2, 3, 4, 5]
        res = filter(lambda x: x % 2 == 0, l)
        assert_eq(res, [2, 4])

        # Filter None (truthy)
        l = [0, 1, False, True]
        res = filter(None, l)
        assert_eq(res, [1, True])
    "#,
    );

    // Reduce
    assert::pass(
        r#"
        l = [1, 2, 3, 4]
        res = reduce(lambda acc, x: acc + x, l)
        assert_eq(res, 10)

        res = reduce(lambda acc, x: acc * x, l, 1)
        assert_eq(res, 24)
    "#,
    );

    assert::fail("reduce(lambda x,y: x, [])", "reduce() of empty sequence");
}

#[test]
fn test_enumerate() {
    assert::pass(
        r#"
        l = ["a", "b"]
        res = []
        for i, x in enumerate(l):
            res.append([i, x])
        assert_eq(res, [[0, "a"], [1, "b"]])

        # Custom start
        res = []
        for i, x in enumerate(l, 10):
            res.append([i, x])
        assert_eq(res, [[10, "a"], [11, "b"]])
    "#,
    );
}

#[test]
fn test_core_builtins() {
    assert::pass("assert(True)");
    assert::fail("assert(False)", "Assertion failed");
    assert::fail("fail('boom')", "boom");

    assert::pass(
        r#"
        assert_eq(len([1, 2]), 2)
        assert_eq(len("abc"), 3)
        assert_eq(len({"a": 1}), 1)
    "#,
    );

    assert::fail("len(1)", "not defined for type");
}
