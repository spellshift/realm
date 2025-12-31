mod assert;

#[test]
fn test_list_extend_edge_cases() {
    // Test extending with Set (currently unsupported, expecting error but future enhancement)
    assert::fail("l = [1]; l.extend({2, 3})", "TypeError");

    // Test extending with String (currently unsupported)
    assert::fail("l = [1]; l.extend('abc')", "TypeError");

    // Test extending with Dict (currently unsupported)
    assert::fail("l = [1]; l.extend({'a': 1})", "TypeError");
}

#[test]
fn test_set_argument_validation() {
    // Pop takes 0 args
    assert::fail(
        "s = {1}; s.pop(1)",
        "TypeError: pop() takes exactly 0 arguments",
    );

    // Clear takes 0 args
    assert::fail(
        "s = {1}; s.clear(1)",
        "TypeError: clear() takes exactly 0 arguments",
    );

    // Add takes 1 arg
    assert::fail(
        "s = {1}; s.add()",
        "TypeError: add() takes exactly 1 argument",
    );
    assert::fail(
        "s = {1}; s.add(1, 2)",
        "TypeError: add() takes exactly 1 argument",
    );
}

#[test]
fn test_list_argument_validation() {
    // Sort takes 0 args
    assert::fail("l = [1]; l.sort(1)", "TypeError");

    // Reverse is not implemented as a method
    assert::fail("l = [1]; l.reverse()", "has no method 'reverse'");
}

#[test]
fn test_dict_argument_validation() {
    // update takes 1 arg
    assert::fail(
        "d = {}; d.update()",
        "TypeError: update() takes exactly 1 argument",
    );
    assert::fail(
        "d = {}; d.update(1, 2)",
        "TypeError: update() takes exactly 1 argument",
    );

    // update with non-dict
    assert::fail("d = {}; d.update([('a', 1)])", "TypeError"); // currently only supports dict
}
