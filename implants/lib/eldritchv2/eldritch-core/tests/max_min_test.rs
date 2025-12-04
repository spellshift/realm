mod assert;

#[test]
fn test_max_basic() {
    // Basic integer comparison
    assert::pass("assert_eq(max(1, 2, 3), 3)");
    assert::pass("assert_eq(max(3, 2, 1), 3)");
    // max(1) should fail because single argument must be iterable
    assert::fail("max(1)", "object is not iterable");
}

#[test]
fn test_max_iterable() {
    // List
    assert::pass("assert_eq(max([1, 2, 3]), 3)");
    assert::pass("assert_eq(max([3, 2, 1]), 3)");

    // Tuple
    assert::pass("assert_eq(max((1, 2, 3)), 3)");

    // Set
    assert::pass("assert_eq(max({1, 2, 3}), 3)");

    // String (codepoint comparison)
    assert::pass("assert_eq(max('abc'), 'c')");

    // Dict (keys)
    assert::pass("assert_eq(max({'a': 1, 'b': 2}), 'b')");
}

#[test]
fn test_max_mixed_args() {
    // Float and Int
    assert::pass("assert_eq(max(1, 2.5, 0), 2.5)");
    assert::pass("assert_eq(max(10.0, 5), 10.0)");
}

#[test]
fn test_max_strings() {
    assert::pass("assert_eq(max('apple', 'banana', 'cherry'), 'cherry')");
}

#[test]
fn test_max_errors() {
    assert::fail("max()", "max expected at least 1 argument");
    assert::fail("max([])", "empty sequence");
}

#[test]
fn test_min_basic() {
    // Basic integer comparison
    assert::pass("assert_eq(min(1, 2, 3), 1)");
    assert::pass("assert_eq(min(3, 2, 1), 1)");
    assert::fail("min(1)", "object is not iterable");
}

#[test]
fn test_min_iterable() {
    // List
    assert::pass("assert_eq(min([1, 2, 3]), 1)");

    // String
    assert::pass("assert_eq(min('abc'), 'a')");
}

#[test]
fn test_min_mixed_args() {
    assert::pass("assert_eq(min(1, 2.5, 5), 1)");
}

#[test]
fn test_min_errors() {
    assert::fail("min()", "min expected at least 1 argument");
    assert::fail("min([])", "empty sequence");
}
