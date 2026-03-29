mod assert;

#[test]
fn test_list_slicing_basic() {
    assert::pass(
        r#"
        l = [0, 1, 2, 3, 4, 5]
        assert_eq(l[0:6], l)
        assert_eq(l[:], l)
        assert_eq(l[0:3], [0, 1, 2])
        assert_eq(l[3:], [3, 4, 5])
        assert_eq(l[:3], [0, 1, 2])
        assert_eq(l[3:6], [3, 4, 5])
    "#,
    );
}

#[test]
fn test_list_slicing_steps() {
    assert::pass(
        r#"
        l = [0, 1, 2, 3, 4, 5]
        assert_eq(l[::2], [0, 2, 4])
        assert_eq(l[1::2], [1, 3, 5])
        assert_eq(l[::3], [0, 3])
        assert_eq(l[::100], [0])
    "#,
    );
}

#[test]
fn test_list_slicing_negative_indices() {
    assert::pass(
        r#"
        l = [0, 1, 2, 3, 4, 5]
        assert_eq(l[-1], 5)
        assert_eq(l[-2], 4)
        assert_eq(l[:-1], [0, 1, 2, 3, 4])
        assert_eq(l[-3:], [3, 4, 5])
        assert_eq(l[-3:-1], [3, 4])
    "#,
    );
}

#[test]
fn test_list_slicing_negative_steps() {
    assert::pass(
        r#"
        l = [0, 1, 2, 3, 4, 5]
        assert_eq(l[::-1], [5, 4, 3, 2, 1, 0])
        assert_eq(l[::-2], [5, 3, 1])
        assert_eq(l[4:2:-1], [4, 3])
        assert_eq(l[2:4:-1], [])
    "#,
    );
}

#[test]
fn test_list_slicing_empty_result_edge_cases() {
    assert::pass(
        r#"
        l = [0, 1, 2, 3, 4, 5]
        # Start > Stop with positive step
        assert_eq(l[4:2], [])
        # Start < Stop with negative step
        assert_eq(l[2:4:-1], [])
        # Out of bounds start (positive)
        assert_eq(l[100:], [])
        # Out of bounds stop (negative)
        assert_eq(l[:-100], [])
    "#,
    );
}

#[test]
fn test_list_slicing_out_of_bounds() {
    assert::pass(
        r#"
        l = [0, 1, 2]
        assert_eq(l[0:100], [0, 1, 2])
        assert_eq(l[-100:], [0, 1, 2])
        assert_eq(l[-100:-50], [])
    "#,
    );
}

#[test]
fn test_string_slicing_extended() {
    assert::pass(
        r#"
        s = "012345"
        assert_eq(s[::2], "024")
        assert_eq(s[::-1], "543210")
        assert_eq(s[-3:], "345")
        assert_eq(s[100:], "")
        assert_eq(s[-100:], "012345")

        # Empty string
        assert_eq(""[:], "")
        assert_eq(""[::-1], "")
    "#,
    );
}

#[test]
fn test_tuple_slicing_extended() {
    assert::pass(
        r#"
        t = (0, 1, 2, 3, 4, 5)
        assert_eq(t[::2], (0, 2, 4))
        assert_eq(t[::-1], (5, 4, 3, 2, 1, 0))
        assert_eq(t[100:], ())
    "#,
    );
}

#[test]
fn test_bytes_slicing() {
    assert::pass(
        r#"
        b = b"012345"
        assert_eq(b[0:6], b)
        assert_eq(b[:], b)
        assert_eq(b[0:3], b"012")
        assert_eq(b[3:], b"345")
        assert_eq(b[:3], b"012")
        assert_eq(b[::2], b"024")
        assert_eq(b[::-1], b"543210")
        assert_eq(b[-3:], b"345")
        assert_eq(b[100:], b"")
        assert_eq(b[-100:], b"012345")
    "#,
    );
}

#[test]
fn test_slicing_zero_step_error() {
    assert::fail("l = [1]; l[::0]", "slice step cannot be zero");
    assert::fail("s = 'a'; s[::0]", "slice step cannot be zero");
    assert::fail("t = (1,); t[::0]", "slice step cannot be zero");
    assert::fail("b = b'a'; b[::0]", "slice step cannot be zero");
}
