mod assert;

#[test]
fn test_any_extended() {
    // Strings
    assert::pass("assert(any('abc') == True)");
    assert::pass("assert(any('') == False)");

    // Lists
    assert::pass("assert(any([False, True, False]) == True)");
    assert::pass("assert(any([False, False]) == False)");
    assert::pass("assert(any([]) == False)");

    // Sets
    assert::pass("assert(any({False, True}) == True)");
    assert::pass("assert(any({False}) == False)");
    assert::pass("assert(any(set()) == False)");
}

#[test]
fn test_all_extended() {
    // Strings
    assert::pass("assert(all('abc') == True)");
    assert::pass("assert(all('') == True)");

    // Lists
    assert::pass("assert(all([True, True]) == True)");
    assert::pass("assert(all([True, False]) == False)");
    assert::pass("assert(all([]) == True)");

    // Sets
    assert::pass("assert(all({True}) == True)");
    assert::pass("assert(all({True, False}) == False)");
    assert::pass("assert(all(set()) == True)");
}

#[test]
fn test_map_extended() {
    // map with different length iterables (not supported yet, checking fail/behavior)
    // Python supports it, stops at shortest. Eldritch map takes exactly 2 args (func, iterable).
    // So we can only test single iterable map for now.

    // Test mapping over string
    assert::pass(
        r#"
        res = map(lambda x: x + x, "abc")
        assert_eq(res, ["aa", "bb", "cc"])
    "#,
    );

    // Test mapping over empty list
    assert::pass(
        r#"
        res = map(lambda x: x, [])
        assert_eq(res, [])
    "#,
    );
}

#[test]
fn test_filter_extended() {
    // Filter string
    assert::pass(
        r#"
        res = filter(lambda x: x != "b", "abc")
        assert_eq(res, ["a", "c"])
    "#,
    );

    // Filter None with string
    assert::pass(
        r#"
        res = filter(None, "abc")
        assert_eq(res, ["a", "b", "c"])
    "#,
    );
}

#[test]
fn test_zip_extended() {
    // Zip strings
    assert::pass(
        r#"
        z = zip("ab", "cd")
        assert_eq(z, [("a", "c"), ("b", "d")])
    "#,
    );

    // Zip string and list
    assert::pass(
        r#"
        z = zip("ab", [1, 2])
        assert_eq(z, [("a", 1), ("b", 2)])
    "#,
    );
}

#[test]
fn test_sorted_extended() {
    // Reverse
    assert::pass(
        r#"
        l = [1, 3, 2]
        res = sorted(l, reverse=True)
        assert_eq(res, [3, 2, 1])
    "#,
    );

    // Key
    assert::pass(
        r#"
        l = ["ccc", "a", "bb"]
        res = sorted(l, key=len)
        assert_eq(res, ["a", "bb", "ccc"])
    "#,
    );

    // Key and Reverse
    assert::pass(
        r#"
        l = ["ccc", "a", "bb"]
        res = sorted(l, key=len, reverse=True)
        assert_eq(res, ["ccc", "bb", "a"])
    "#,
    );

    // Sorting string (returns list of chars)
    assert::pass(
        r#"
        s = "bca"
        res = sorted(s)
        assert_eq(res, ["a", "b", "c"])
    "#,
    );
}

#[test]
fn test_max_min_extended() {
    // String
    assert::pass("assert(max('abc') == 'c')");
    assert::pass("assert(min('abc') == 'a')");
}
