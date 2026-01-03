mod assert;

#[test]
fn test_set_intersection_operator() {
    assert::pass(
        r#"
        s1 = {1, 2, 3}
        s2 = {2, 3, 4}
        assert_eq(s1 & s2, {2, 3})
        assert_eq(s2 & s1, {2, 3})
        assert_eq(s1 & set(), set())
    "#,
    );
}

#[test]
fn test_set_union_operator() {
    assert::pass(
        r#"
        s1 = {1, 2}
        s2 = {2, 3}
        assert_eq(s1 | s2, {1, 2, 3})
        assert_eq(s2 | s1, {1, 2, 3})
        assert_eq(s1 | set(), {1, 2})
    "#,
    );
}

#[test]
fn test_set_symmetric_difference_operator() {
    assert::pass(
        r#"
        s1 = {1, 2, 3}
        s2 = {2, 3, 4}
        assert_eq(s1 ^ s2, {1, 4})
        assert_eq(s2 ^ s1, {1, 4})
        assert_eq(s1 ^ set(), {1, 2, 3})
        assert_eq(s1 ^ s1, set())
    "#,
    );
}

#[test]
fn test_set_difference_operator() {
    assert::pass(
        r#"
        s1 = {1, 2, 3}
        s2 = {2, 3, 4}
        assert_eq(s1 - s2, {1})
        assert_eq(s2 - s1, {4})
        assert_eq(s1 - set(), {1, 2, 3})
        assert_eq(set() - s1, set())
    "#,
    );
}

#[test]
fn test_set_inplace_operators() {
    // Only += works in-place for sets due to limited parser/exec support for now.
    // &=, |=, -=, ^= are not standard AugmentedAssignment in Eldritch v2 implementation yet (only +=, -=, *=, /=, //=, %=).
    // wait, looking at TokenKind, there are no &=, |=, ^= tokens.
    // PlusAssign, MinusAssign, StarAssign, SlashAssign, PercentAssign, SlashSlashAssign.
    // So only += is supported.

    // Note: MinusAssign -= exists in token.rs and Lexer.
    // Let's verify if -= works.
    assert::pass(
        r#"
        s = {1, 2}
        s -= {1}
        # Sets don't support -= in try_inplace_add, so it falls back to binary op which returns new set.
        assert_eq(s, {2})
    "#,
    );
}

#[test]
fn test_dict_union_operator() {
    assert::pass(
        r#"
        d1 = {"a": 1, "b": 2}
        d2 = {"b": 3, "c": 4}
        # d2 overwrites d1
        assert_eq(d1 | d2, {"a": 1, "b": 3, "c": 4})
    "#,
    );
}

#[test]
fn test_set_operator_type_errors() {
    assert::fail("{1} & 1", "unsupported operand type(s)");
    assert::fail("{1} | 1", "unsupported operand type(s)");
    assert::fail("{1} ^ 1", "unsupported operand type(s)");
    assert::fail("{1} - 1", "unsupported operand type(s)");
}
