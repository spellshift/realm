mod assert;

#[test]
fn test_floor_division() {
    assert::all_true(
        r#"
        10 // 2 == 5
        10 // 3 == 3
        -10 // 3 == -4
        10 // -3 == -4
        -10 // -3 == 3
        1 // 2 == 0
        -1 // 2 == -1
    "#,
    );
    assert::fail("1 // 0", "divide by zero");
}

#[test]
fn test_augmented_assignment() {
    assert::pass(
        r#"
        a = 1
        a += 2
        assert(a == 3)

        a -= 1
        assert(a == 2)

        a *= 3
        assert(a == 6)

        a //= 2
        assert(a == 3)

        a /= 3
        assert(a == 1)

        a %= 1
        assert(a == 0)
    "#,
    );
}

#[test]
fn test_augmented_assignment_list() {
    assert::pass(
        r#"
        l = [1, 2, 3]
        l[0] += 1
        assert(l[0] == 2)
        assert(l == [2, 2, 3])
    "#,
    );
}

#[test]
fn test_augmented_assignment_dict() {
    assert::pass(
        r#"
        d = {"a": 1}
        d["a"] += 1
        assert(d["a"] == 2)
        assert(d == {"a": 2})
    "#,
    );
}

#[test]
fn test_ternary_if() {
    assert::all_true(
        r#"
        (1 if True else 2) == 1
        (1 if False else 2) == 2
        ("yes" if 1 < 2 else "no") == "yes"
    "#,
    );
}

#[test]
fn test_string_modulo() {
    assert::all_true(
        r#"
        "Hello %s" % "World" == "Hello World"
        "Value: %s" % 10 == "Value: 10"
        "%s %s" % ("a", "b") == "a b"
        "%s %%" % 100 == "100 %"
    "#,
    );
    assert::fail("'Hello %s' % ()", "not enough arguments"); // No args for %s
    assert::fail("'Hello' % 'World'", "not all arguments converted"); // Extra args
}

#[test]
fn test_unpacking_assignment() {
    assert::pass(
        r#"
        a, b = 1, 2
        assert(a == 1)
        assert(b == 2)

        x, y = [3, 4]
        assert(x == 3)
        assert(y == 4)

        # Swap
        a, b = b, a
        assert(a == 2)
        assert(b == 1)
    "#,
    );

    assert::fail("a, b = 1", "cannot unpack non-iterable");
    assert::fail("a, b = [1]", "not enough values to unpack");
    assert::fail("a, b = [1, 2, 3]", "too many/not enough values");
}
