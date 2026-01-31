mod assert;

#[test]
fn test_dict_iteration() {
    assert::pass(
        r#"
        d = {"a": 1, "b": 2}

        # Test basic iteration (should yield keys)
        keys = []
        for k in d:
            keys.append(k)

        # Sort keys to ensure deterministic order check
        keys.sort()
        assert_eq(keys, ["a", "b"])

        # Test iteration in list comprehension
        keys_comp = [k for k in d]
        keys_comp.sort()
        assert_eq(keys_comp, ["a", "b"])
    "#,
    );
}

#[test]
fn test_dict_iteration_empty() {
    assert::pass(
        r#"
        d = {}
        count = 0
        for k in d:
            count += 1
        assert_eq(count, 0)
    "#,
    );
}

#[test]
fn test_dict_iteration_mutation() {
    // It's generally unsafe to modify a collection while iterating,
    // but we should verify behavior (e.g. no crash).
    // In our implementation (items collected to vector before loop), mutation should not affect the current loop.
    assert::pass(
        r#"
        d = {"a": 1, "b": 2}
        keys = []
        for k in d:
            keys.append(k)
            if k == "a":
                d["c"] = 3 # Should not be visited this loop

        keys.sort()
        assert_eq(keys, ["a", "b"])
        assert_eq(d["c"], 3)
    "#,
    );
}
