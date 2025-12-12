mod assert;

#[test]
fn test_bitwise_not() {
    assert::pass(r#"
        assert_eq(~1, -2)
        assert_eq(~0, -1)
        assert_eq(~-1, 0)
    "#);
    assert::fail("~1.0", "only valid for integers");
}

#[test]
fn test_eval_index_out_of_bounds() {
    // List
    assert::fail("[][0]", "List index out of range");
    assert::fail("[1][1]", "List index out of range");
    assert::fail("[1][-2]", "List index out of range");

    // Tuple
    assert::fail("()[0]", "Tuple index out of range");
    assert::fail("(1,)[1]", "Tuple index out of range");
    assert::fail("(1,)[-2]", "Tuple index out of range");

    // String
    // Eldritch doesn't seem to support string indexing in eval.rs yet?
    // Wait, evaluate_index only handles List, Tuple, Dictionary.
    // Let's verify this failure.
    assert::fail("'abc'[0]", "'string' object is not subscriptable");
}

#[test]
fn test_string_format_errors() {
    assert::fail("'%d' % 's'", "%d format: a number is required");
    assert::fail("'%s' % (1, 2)", "not all arguments converted");
    assert::fail("'%' % 1", "incomplete format");
    assert::fail("'%q' % 1", "unsupported format character 'q'");
}

#[test]
fn test_nested_comparisons() {
    assert::pass(r#"
        assert([1, [2, 3]] < [1, [2, 4]])
        assert([1, [2, 3]] == [1, [2, 3]])
        assert([1, (1, 2)] != [1, (1, 3)])
    "#);
}

#[test]
fn test_augmented_assignment_edge_cases() {
    // Test for list * int in place?
    // Current `try_inplace_add` only handles + for List, Dict, Set.
    // So `l *= 2` will go through `apply_binary_op` (which creates new list) and then assign back.
    assert::pass(r#"
        l = [1]
        l *= 2
        assert_eq(l, [1, 1])
    "#);

    // In-place add for sets
    assert::pass(r#"
        s = {1}
        s += {2}
        assert_eq(s, {1, 2})
    "#);
}

#[test]
fn test_augmented_assignment_dict() {
    assert::pass(r#"
        d = {"a": 1}
        d += {"b": 2}
        assert_eq(d, {"a": 1, "b": 2})
    "#);
}
