mod assert;

#[test]
fn test_list_comp_basic() {
    assert::pass(
        r#"
        l = [x * 2 for x in [1, 2, 3]]
        assert(l == [2, 4, 6])
    "#,
    );
}

#[test]
fn test_list_comp_with_if() {
    assert::pass(
        r#"
        l = [x for x in [1, 2, 3, 4] if x % 2 == 0]
        assert(l == [2, 4])
    "#,
    );
}

#[test]
fn test_dict_comp_basic() {
    assert::pass(
        r#"
        d = {x: x * 2 for x in [1, 2]}
        assert(d[1] == 2)
        assert(d[2] == 4)
        assert(len(d) == 2)
    "#,
    );
}

#[test]
fn test_dict_comp_with_if() {
    assert::pass(
        r#"
        d = {x: x for x in [1, 2, 3, 4] if x > 2}
        assert(len(d) == 2)
        assert(d[3] == 3)
        assert(d[4] == 4)
    "#,
    );
}

#[test]
fn test_set_comp_basic() {
    assert::pass(
        r#"
        s = {x % 2 for x in [1, 2, 3, 4]}
        assert(s == {0, 1})
    "#,
    );
}

#[test]
fn test_set_comp_with_if() {
    assert::pass(
        r#"
        s = {x for x in [1, 2, 3] if x > 1}
        assert(s == {2, 3})
    "#,
    );
}

#[test]
fn test_comp_scope_leak() {
    // Verify variable 'x' leaks or not?
    // In Python 3, list comp variables do NOT leak.
    // In Eldritch, they currently DO NOT leak because we use a new scope.
    assert::pass(
        r#"
        x = 100
        l = [x for x in [1, 2]]
        assert(x == 100)
    "#,
    );
}

#[test]
fn test_comp_nested_scope() {
    // Variable from outer scope should be visible
    assert::pass(
        r#"
        y = 5
        l = [x + y for x in [1, 2]]
        assert(l == [6, 7])
    "#,
    );
}

#[test]
fn test_comp_iterating_string() {
    assert::pass(
        r#"
        l = [c.upper() for c in "abc"]
        assert(l == ["A", "B", "C"])
    "#,
    );
}

#[test]
fn test_comp_iterating_dict() {
    assert::pass(
        r#"
        d = {"a": 1, "b": 2}
        l = [k for k in d]
        # order is not guaranteed in dict keys usually, but BTreeMap sorts them
        assert(l == ["a", "b"])
    "#,
    );
}
