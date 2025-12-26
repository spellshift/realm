mod assert;

#[test]
fn test_comp_nested_comprehension_manual() {
    assert::pass(
        r#"
        # Check if we can simulate nested loops via nesting
        l = [[x * y for x in [1, 2]] for y in [1, 2]]
        assert(l == [[1, 2], [2, 4]])
    "#,
    );
}

#[test]
fn test_comp_error_in_iter() {
    assert::fail(
        r#"
        [x for x in 123]
    "#,
        "Type '\"int\"' is not iterable",
    );
}

#[test]
fn test_comp_error_in_cond() {
    assert::fail(
        r#"
        [x for x in [1] if "a" + 1]
    "#,
        "unsupported operand type(s) for +: 'string' and 'int'",
    );
}

#[test]
fn test_comp_error_in_body() {
    assert::fail(
        r#"
        [x + "a" for x in [1]]
    "#,
        "unsupported operand type(s) for +: 'int' and 'string'",
    );
}

#[test]
fn test_comp_shadowing() {
    assert::pass(
        r#"
        x = [1, 2]
        # x inside is the loop var, x in [x] refers to outer x
        l = [x for x in x]
        assert(l == [1, 2])
    "#,
    );
}
