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

#[test]
fn test_dir_scope() {
    // Test that dir() sees variables in current and parent scopes
    assert::pass(
        r#"
        x = 10
        d = dir()
        assert("x" in d)
        assert("print" in d) # built-in
    "#,
    );

    assert::pass(
        r#"
        global_var = 1
        def foo():
            local_var = 2
            d = dir()
            assert("local_var" in d)
            assert("global_var" in d)
            assert("print" in d)

        foo()
    "#,
    );
}

#[test]
fn test_libs_builtins() {
    assert::pass(
        r#"
        l = libs()
        # We can't guarantee what libs are loaded in test env, but it should return a list
        assert(type(l) == "list")
    "#,
    );

    assert::pass(
        r#"
        b = builtins()
        assert("print" in b)
        assert("dir" in b)
        assert("libs" in b)
        assert("builtins" in b)
    "#,
    );
}

#[test]
fn test_bytes_builtin() {
    // String to bytes
    assert::pass(
        r#"
        b = bytes("hello")
        assert(type(b) == "bytes")
        assert_eq(len(b), 5)
    "#,
    );

    // List of ints to bytes
    assert::pass(
        r#"
        b = bytes([65, 66, 67])
        assert(type(b) == "bytes")
        assert_eq(b, bytes("ABC"))
    "#,
    );

    // Int to bytes (zero-filled)
    assert::pass(
        r#"
        b = bytes(5)
        assert(type(b) == "bytes")
        assert_eq(len(b), 5)
        assert_eq(b, bytes([0, 0, 0, 0, 0]))
    "#,
    );

    // Errors
    assert::fail("bytes(-1)", "negative");
    assert::fail("bytes([256])", "range");
    assert::fail("bytes(['a'])", "integers");
}

#[test]
fn test_type_regression() {
    assert::pass(r#"
        assert(type(1) == "int")
    "#);
    assert::fail("type()", "expects exactly one argument");
    assert::fail("type(1, 2)", "expects exactly one argument");
}

#[test]
fn test_min_max_args() {
    // Max with multiple args
    assert::pass(
        r#"
        assert_eq(max(1, 2), 2)
        assert_eq(max(2, 1), 2)
        assert_eq(max(1, 2, 3), 3)
        assert_eq(max(3, 2, 1), 3)
        assert_eq(max(1, 3, 2), 3)
    "#);

    // Min with multiple args
    assert::pass(
        r#"
        assert_eq(min(1, 2), 1)
        assert_eq(min(2, 1), 1)
        assert_eq(min(1, 2, 3), 1)
        assert_eq(min(3, 2, 1), 1)
        assert_eq(min(1, 3, 2), 1)
    "#);

    // Existing behavior (iterable)
    assert::pass(
        r#"
        assert_eq(max([1, 2, 3]), 3)
        assert_eq(min([1, 2, 3]), 1)
    "#);

    // Mixed types (float vs int)
    assert::pass(
        r#"
        assert_eq(max(1, 2.5), 2.5)
        assert_eq(min(1, 2.5), 1)
    "#);

    // Errors
    assert::fail("max()", "expected at least 1 argument");
    assert::fail("min()", "expected at least 1 argument");
    // existing error check
    assert::fail("max(1)", "not iterable");
    assert::fail("min(1)", "not iterable");
}
