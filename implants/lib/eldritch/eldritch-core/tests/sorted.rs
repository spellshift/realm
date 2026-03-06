mod assert;

#[test]
fn test_sorted_coverage() {
    assert::pass(
        r#"
        # Basic sorting
        assert_eq(sorted([3, 1, 2]), [1, 2, 3])
        assert_eq(sorted((3, 1, 2)), [1, 2, 3])
        assert_eq(sorted("cba"), ["a", "b", "c"])

        # Reverse sort
        assert_eq(sorted([1, 2, 3], reverse=True), [3, 2, 1])
        assert_eq(sorted([3, 1, 2], reverse=False), [1, 2, 3])

        # Key sorting
        assert_eq(sorted(["a", "ccc", "bb"], key=len), ["a", "bb", "ccc"])
        assert_eq(sorted(["a", "ccc", "bb"], key=len, reverse=True), ["ccc", "bb", "a"])

        # None key acts like no key
        assert_eq(sorted([3, 1, 2], key=None), [1, 2, 3])

        # Custom function for key
        def get_second(item):
            return item[1]

        assert_eq(sorted([[1, 3], [2, 1], [3, 2]], key=get_second), [[2, 1], [3, 2], [1, 3]])
        "#,
    );
}

#[test]
fn test_sorted_errors() {
    assert::fail("sorted()", "missing 1 required positional argument");

    assert::fail(
        "sorted([1, 2], [3, 4])",
        "sorted() takes only 1 positional argument",
    );

    assert::fail(
        "sorted([1], unknown=True)",
        "sorted() got an unexpected keyword argument 'unknown'",
    );

    assert::fail(
        "sorted(*[1])",
        "sorted() does not support *args or **kwargs unpacking",
    );

    assert::fail(
        "sorted(**{'a': 1})",
        "sorted() does not support *args or **kwargs unpacking",
    );
}
