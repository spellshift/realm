mod assert;

#[test]
fn test_comparison_edge_cases() {
    // NaN comparisons
    // In IEEE 754, NaN != NaN.
    assert::pass(
        r#"
        nan = float("nan")
        assert(nan != nan)
        assert(not (nan == nan))

        # Comparison with other types
        assert(nan != 1)
        assert(nan != "s")
    "#,
    );

    // Mixed Int/Float equality
    assert::pass(
        r#"
        assert(1 == 1.0)
        assert(1.0 == 1)
        assert(not (1 != 1.0))
        assert(not (1.0 != 1))

        assert(1 != 1.1)
        assert(1.1 != 1)
    "#,
    );

    // Incompatible type comparisons
    // Currently implementation returns False for equality checks between different types (except numeric),
    // but raises TypeError for ordering (<, <=, >, >=).

    assert::pass(
        r#"
        assert("1" != 1)
        assert(1 != "1")
        assert(not ("1" == 1))
        assert(not (1 == "1"))
    "#,
    );

    assert::fail("1 < '1'", "not supported between instances of 'int' and 'string'");
    assert::fail("'1' > 1", "not supported between instances of 'string' and 'int'");
    assert::fail("1 <= []", "not supported between instances of 'int' and 'list'");
    assert::fail("{} >= 1", "not supported between instances of 'dict' and 'int'");
}

#[test]
fn test_complex_comparisons() {
    // Chained comparisons are handled by the parser/compiler as separate assertions usually,
    // but if implemented as `a < b < c` -> `a < b and b < c`, let's verify.
    // Assuming the language supports chained comparisons or at least simple ones work.

    assert::pass(
        r#"
        assert(1 < 2)
        assert(2 > 1)
        assert(1 <= 1)
        assert(1 >= 1)
    "#,
    );
}
