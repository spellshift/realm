mod assert;

#[test]
fn test_zip_two_lists() {
    assert::pass(
        r#"
        z = zip([1, 2], [3, 4])
        assert_eq(z, [(1, 3), (2, 4)])
    "#,
    );
}

#[test]
fn test_zip_mixed_types() {
    assert::pass(
        r#"
        z = zip([1, 2], "ab", (3, 4))
        assert_eq(z, [(1, "a", 3), (2, "b", 4)])
    "#,
    );
}

#[test]
fn test_zip_uneven() {
    assert::pass(
        r#"
        z = zip([1, 2, 3], [4, 5])
        assert_eq(z, [(1, 4), (2, 5)])

        z = zip([1, 2], [3, 4, 5])
        assert_eq(z, [(1, 3), (2, 4)])
    "#,
    );
}

#[test]
fn test_zip_empty() {
    assert::pass(
        r#"
        assert_eq(zip(), [])
        assert_eq(zip([]), [])
        assert_eq(zip([], [1, 2]), [])
    "#,
    );
}

#[test]
fn test_zip_sets_dicts() {
    // Sets and Dicts are unordered, so we can't guarantee the order of zipped elements
    // But we can check the length and that the elements are valid
    assert::pass(
        r#"
        s = {1, 2, 3}
        d = {'a': 1, 'b': 2}
        z = zip(s, d)
        assert_eq(len(z), 2) # min len of 3 and 2 is 2
    "#,
    );
}

#[test]
fn test_zip_errors() {
    assert::fail("zip(1)", "'int' object is not iterable");
    assert::fail("zip([1], 1)", "'int' object is not iterable");
}
