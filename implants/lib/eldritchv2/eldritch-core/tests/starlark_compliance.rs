mod assert;

#[test]
fn test_list_multiplication() {
    assert::pass(
        r#"
        l = [1, 2]
        assert_eq(l * 3, [1, 2, 1, 2, 1, 2])
        assert_eq(l * 0, [])
        assert_eq(l * -1, [])

        # Commutative?
        assert_eq(3 * l, [1, 2, 1, 2, 1, 2])
    "#,
    );
}

#[test]
fn test_tuple_multiplication() {
    assert::pass(
        r#"
        t = (1, 2)
        assert_eq(t * 2, (1, 2, 1, 2))
        assert_eq(t * 0, ())
    "#,
    );
}

#[test]
fn test_string_multiplication() {
    assert::pass(
        r#"
        s = "ab"
        assert_eq(s * 3, "ababab")
        assert_eq(3 * s, "ababab")
        assert_eq(s * 0, "")
    "#,
    );
}

#[test]
fn test_sequence_comparison() {
    assert::pass(
        r#"
        assert([1, 2] < [1, 3])
        assert([1, 2] > [1, 1])
        assert([1, 2] == [1, 2])
        assert([1, 2] != [1, 3])

        assert((1, 2) < (1, 3))
        assert((1, 2) > (1, 1))
    "#,
    );
}

#[test]
fn test_extended_string_formatting() {
    assert::pass(
        r#"
        assert_eq("Val: %d" % 10, "Val: 10")
        assert_eq("Repr: %r" % "s", "Repr: \"s\"")
    "#,
    );
}
