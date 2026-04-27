mod assert;

#[test]
fn test_bytes_split() {
    assert::pass(
        r#"
        b = b"a,b,c"
        assert_eq(b.split(b","), [b"a", b"b", b"c"])
        assert_eq(b"  a  b  ".split(), [b"a", b"b"])
        assert_eq(b"a--b--c".split(b"--"), [b"a", b"b", b"c"])
        # Edge case: delim longer than bytes
        assert_eq(b"abc".split(b"abcdef"), [b"abc"])
    "#,
    );
}

#[test]
fn test_bytes_splitlines() {
    assert::pass(
        r#"
        b = b"line1\nline2\r\nline3"
        assert_eq(b.splitlines(), [b"line1", b"line2", b"line3"])
        assert_eq(b.splitlines(True), [b"line1\n", b"line2\r\n", b"line3"])
    "#,
    );
}

#[test]
fn test_bytes_rsplit() {
    assert::pass(
        r#"
        b = b"a,b,c"
        assert_eq(b.rsplit(b","), [b"a", b"b", b"c"])
        assert_eq(b"  a  b  ".rsplit(), [b"a", b"b"])
        # Edge case: delim longer than bytes
        assert_eq(b"abc".rsplit(b"abcdef"), [b"abc"])
    "#,
    );
}

#[test]
fn test_bytes_strip() {
    assert::pass(
        r#"
        assert_eq(b"  abc  ".strip(), b"abc")
        assert_eq(b"xxabcyy".strip(b"xy"), b"abc")
        assert_eq(b"  abc  ".lstrip(), b"abc  ")
        assert_eq(b"xxabcyy".lstrip(b"x"), b"abcyy")
        assert_eq(b"  abc  ".rstrip(), b"  abc")
        assert_eq(b"xxabcyy".rstrip(b"y"), b"xxabc")
    "#,
    );
}

#[test]
fn test_bytes_startswith_endswith() {
    assert::pass(
        r#"
        b = b"hello"
        assert(b.startswith(b"he"))
        assert(not b.startswith(b"ho"))
        assert(b.endswith(b"lo"))
        assert(not b.endswith(b"la"))
    "#,
    );
}

#[test]
fn test_bytes_removeprefix_removesuffix() {
    assert::pass(
        r#"
        b = b"hello"
        assert_eq(b.removeprefix(b"he"), b"llo")
        assert_eq(b.removeprefix(b"lo"), b"hello")
        assert_eq(b.removesuffix(b"lo"), b"hel")
        assert_eq(b.removesuffix(b"he"), b"hello")
    "#,
    );
}

#[test]
fn test_bytes_find_index() {
    assert::pass(
        r#"
        b = b"hello"
        assert_eq(b.find(b"l"), 2)
        assert_eq(b.find(b"z"), -1)
        assert_eq(b.index(b"l"), 2)
        assert_eq(b.rfind(b"l"), 3)
        assert_eq(b.rindex(b"l"), 3)
        # Edge cases: pattern longer than bytes
        assert_eq(b"abc".find(b"abcdef"), -1)
        assert_eq(b"abc".rfind(b"abcdef"), -1)
    "#,
    );
}

#[test]
fn test_bytes_count() {
    assert::pass(
        r#"
        b = b"hello"
        assert_eq(b.count(b"l"), 2)
        assert_eq(b.count(b"o"), 1)
        assert_eq(b.count(b"z"), 0)
        # Edge case: pattern longer than bytes
        assert_eq(b"abc".count(b"abcdef"), 0)
    "#,
    );
}

#[test]
fn test_bytes_replace() {
    assert::pass(
        r#"
        b = b"hello"
        assert_eq(b.replace(b"l", b"p"), b"heppo")
        assert_eq(b.replace(b"he", b"ha"), b"hallo")
        # Edge case: old longer than bytes
        assert_eq(b"abc".replace(b"abcdef", b"x"), b"abc")
    "#,
    );
}

#[test]
fn test_bytes_join() {
    assert::pass(
        r#"
        sep = b","
        assert_eq(sep.join([b"a", b"b", b"c"]), b"a,b,c")
    "#,
    );
}

#[test]
fn test_bytes_partition() {
    assert::pass(
        r#"
        b = b"a,b,c"
        assert_eq(b.partition(b","), (b"a", b",", b"b,c"))
        assert_eq(b.rpartition(b","), (b"a,b", b",", b"c"))
        # Edge cases: sep longer than bytes
        assert_eq(b"abc".partition(b"abcdef"), (b"abc", b"", b""))
        assert_eq(b"abc".rpartition(b"abcdef"), (b"", b"", b"abc"))
    "#,
    );
}
