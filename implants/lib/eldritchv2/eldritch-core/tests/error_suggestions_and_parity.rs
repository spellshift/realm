mod assert;

#[test]
fn test_error_suggestions() {
    // List suggestions
    assert::fail("l = [1]; l.apend(2)", "Did you mean 'append'?");
    assert::fail("l = [1]; l.instert(0, 1)", "Did you mean 'insert'?");
    assert::fail("l = [1]; l.remve(1)", "Did you mean 'remove'?");
    assert::fail("l = [1]; l.sot()", "Did you mean 'sort'?");

    // Dict suggestions
    assert::fail("d = {'a': 1}; d.vaules()", "Did you mean 'values'?");
    assert::fail("d = {'a': 1}; d.udate({'b': 2})", "Did you mean 'update'?");
    assert::fail("d = {'a': 1}; d.popitm()", "Did you mean 'popitem'?");

    // Set suggestions
    assert::fail("s = {1}; s.addd(2)", "Did you mean 'add'?");
    assert::fail("s = {1}; s.disard(1)", "Did you mean 'discard'?");
    assert::fail(
        "s = {1}; s.intersetion({2})",
        "Did you mean 'intersection'?",
    );
    assert::fail("s = {1}; s.unon({2})", "Did you mean 'union'?");

    // String suggestions
    assert::fail("s = 'abc'; s.splti()", "Did you mean 'split'?");
    assert::fail("s = 'abc'; s.strip_()", "Did you mean 'strip'?");
    assert::fail("s = 'abc'; s.uper()", "Did you mean 'upper'?");
    assert::fail("s = 'abc'; s.lowr()", "Did you mean 'lower'?");
    assert::fail("s = 'abc'; s.startwith('a')", "Did you mean 'startswith'?");
}

#[test]
fn test_string_parity_checks() {
    // replace argument count
    assert::pass("s = 'aba'; assert_eq(s.replace('a', 'b'), 'bbb')");
    // Should fail if not exactly 2 args
    assert::fail("s = 'aba'; s.replace('a')", "takes exactly 2 arguments");
    assert::fail(
        "s = 'aba'; s.replace('a', 'b', 1)",
        "takes exactly 2 arguments",
    );

    // index failure
    assert::pass("s = 'abc'; assert_eq(s.index('b'), 1)");
    assert::fail("s = 'abc'; s.index('z')", "ValueError: substring not found");

    // rindex failure
    assert::pass("s = 'abca'; assert_eq(s.rindex('a'), 3)");
    assert::fail(
        "s = 'abc'; s.rindex('z')",
        "ValueError: substring not found",
    );

    // count
    assert::pass("s = 'banana'; assert_eq(s.count('a'), 3)");
    assert::pass("s = 'banana'; assert_eq(s.count('na'), 2)");
    // Overlapping - Rust matches are non-overlapping, Python is non-overlapping.
    // 'aaaa'.count('aa') -> Python: 2. Rust: 2.
    assert::pass("s = 'aaaa'; assert_eq(s.count('aa'), 2)");
    // Empty string count -> N+1
    assert::pass("s = 'abc'; assert_eq(s.count(''), 4)");

    // join
    assert::pass("s = '-'; assert_eq(s.join(['a', 'b']), 'a-b')");
    // Invalid args
    assert::fail("s = '-'; s.join(1)", "TypeError: join() expects a list");
    assert::fail(
        "s = '-'; s.join(['a', 1])",
        "TypeError: join() expects list of strings",
    );
}
