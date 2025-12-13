mod assert;

#[test]
fn test_mixed_arithmetic_helpers() {
    assert::all_true(
        r#"
        1 + 2.0 == 3.0
        1.5 + 2 == 3.5
        10 - 2.5 == 7.5
        5.5 - 2 == 3.5
        2 * 1.5 == 3.0
        2.5 * 2 == 5.0
        10 / 2.0 == 5.0
        10.0 / 2 == 5.0
    "#,
    );
}

#[test]
fn test_floor_div_mixed() {
    assert::all_true(
        r#"
        10 // 3 == 3
        10.0 // 3 == 3.0
        10 // 3.0 == 3.0
        10.0 // 3.0 == 3.0
        -10 // 3 == -4
        -10.0 // 3 == -4.0
    "#,
    );
}

#[test]
fn test_modulo_mixed() {
    assert::all_true(
        r#"
        10 % 3 == 1
        10.0 % 3 == 1.0
        10 % 3.0 == 1.0
        10.0 % 3.0 == 1.0
        -10 % 3 == 2
        -10.0 % 3 == 2.0
    "#,
    );
}

#[test]
fn test_sequence_comparison_recursive() {
    assert::all_true(
        r#"
        [1, 2] < [1, 3]
        [1, 2] < [1, 2, 0]
        [1, 2] == [1, 2]
        [1, 2] != [1, 3]

        # Nested
        [[1], [2]] < [[1], [3]]
        [[1], [2]] < [[2], [1]]
    "#,
    );
}

#[test]
fn test_mixed_equality_helper() {
    assert::all_true(
        r#"
        1 == 1.0
        1.0 == 1
        1 != 1.1
        1.1 != 1
    "#,
    );
}

#[test]
fn test_comparison_helper_logic() {
    // Tests specifically targeting apply_comparison_op logic
    assert::all_true(
        r#"
        1 < 1.1
        1.1 > 1
        1 <= 1.0
        1.0 >= 1
    "#,
    );
}
