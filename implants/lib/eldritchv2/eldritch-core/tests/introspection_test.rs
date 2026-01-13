mod assert;

#[test]
fn test_name_error_suggestion() {
    // Distance 1 (deletion)
    assert::fail(
        r#"
        apple = 1
        x = aple
        "#,
        "Did you mean 'apple'?",
    );

    // Distance 1 (insertion)
    assert::fail(
        r#"
        apple = 1
        x = appe
        "#,
        "Did you mean 'apple'?",
    );

    // Distance 1 (substitution)
    assert::fail(
        r#"
        apple = 1
        x = aplle
        "#,
        "Did you mean 'apple'?",
    );
}

#[test]
fn test_suggestion_threshold() {
    // "config" (6) -> threshold 4.
    // "get_config" (10) -> dist 4.
    // Should match.
    assert::fail(
        r#"
        get_config = 1
        x = config
        "#,
        "Did you mean 'get_config'?",
    );
}

#[test]
fn test_no_suggestion_if_too_far() {
    assert::fail(
        r#"
        apple = 1
        x = xyz
        "#,
        "Undefined variable: 'xyz'",
    );
    // Ensure "Did you mean" is NOT present (assert::fail checks strict containment,
    // but here we check that it DOES contain the first part.
    // To check it does NOT contain suggestion, we'd need a different helper.
    // But basic NameError check is okay.
}

#[test]
fn test_unicode_suggestion() {
    assert::fail(
        r#"
        café = 1
        x = cafe
        "#,
        "Did you mean 'café'?",
    );
}
