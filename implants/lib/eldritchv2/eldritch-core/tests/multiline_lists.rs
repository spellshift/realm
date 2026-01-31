mod assert;

#[test]
fn test_multiline_list() {
    assert::pass(
        r#"
        my_list = ["a",
                 "b",
                 "c",
                 ]
        assert_eq(len(my_list), 3)
        assert_eq(my_list[0], "a")
        assert_eq(my_list[1], "b")
        assert_eq(my_list[2], "c")
    "#,
    );
}

#[test]
fn test_multiline_list_variants() {
    assert::pass(
        r#"
        # No element on first line
        l1 = [
            1, 2
        ]
        assert_eq(len(l1), 2)

        # Element on last line
        l2 = [
            1,
            2]
        assert_eq(len(l2), 2)

        # Trailing comma, no subsequent element
        l3 = [
            1,
            2,
        ]
        assert_eq(len(l3), 2)

        # No trailing comma
        l4 = [
            1,
            2
        ]
        assert_eq(len(l4), 2)
    "#,
    );
}

#[test]
fn test_multiline_list_comments() {
    assert::pass(
        r#"
        l = [
            1, # comment
            2
        ]
        assert_eq(len(l), 2)
    "#,
    );
}

#[test]
fn test_weird_indentation() {
    assert::pass(
        r#"
        x = [
          1,
        2
        ]
        assert_eq(len(x), 2)
    "#,
    );
}

#[test]
fn test_whitespace_before_newline() {
    // Note the space after the comma
    assert::pass("x = [1, \n2]\nassert_eq(len(x), 2)");
}
