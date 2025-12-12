mod assert;

#[test]
fn test_literals_and_constants() {
    assert::all_true(
        r#"
        True == True
        False == False
        True != False
        None == None
        None != False
        None != True
        None != 0
        None != ""
        None != []
        1 == 1
    "#,
    );
}

#[test]
fn test_basic_arithmetic() {
    assert::all_true(
        r#"
        1 + 2 == 3
        10 - 2 == 8
        5 * 5 == 25
        10 / 2 == 5
        -5 + 3 == -2
        -(5 + 5) == -10
        1 + 2 * 3 == 7
        (1 + 2) * 3 == 9
        10 - 5 - 2 == 3
        10 / 2 * 3 == 15
    "#,
    );
}

#[test]
fn test_bitwise_operators() {
    assert::all_true(
        r#"
        (1 & 2) == 0
        (1 | 2) == 3
        (1 ^ 3) == 2
        ~0 == -1
        (1 << 2) == 4
        (8 >> 1) == 4
    "#,
    );
}

#[test]
fn test_comparisons() {
    assert::all_true(
        r#"
        1 < 2
        2 > 1
        1 <= 1
        1 <= 2
        2 >= 2
        3 >= 2
        1 != 2
        1 == 1
        "a" < "b"
    "#,
    );
}

#[test]
fn test_logic_operators() {
    assert::all_true(
        r#"
        True and True
        not False
        (True or False) == True
        (False or False) == False
        not (1 == 2)
        (True or (1/0)) == True
    "#,
    );
    // Short-circuit check for 'and'
    assert::pass("assert((False and (1/0)) == False)");
}

#[test]
fn test_type_conversions() {
    assert::all_true(
        r#"
        int("123") == 123
        str(123) == "123"
        bool(1) == True
        bool(0) == False
        bool("a") == True
        bool("") == False
        bool([]) == False
        bool([1]) == True
    "#,
    );
}

#[test]
fn test_introspection() {
    assert::all_true(
        r#"
        type(1) == "int"
        type("s") == "string"
        type(True) == "bool"
        type(None) == "NoneType"
        type([]) == "list"
        type({}) == "dict"
        type(()) == "tuple"
    "#,
    );
}

#[test]
fn test_basic_errors() {
    assert::fail("1 / 0", "divide by zero");
    assert::fail("undefined_var", "Undefined variable");
    assert::fail("1 + 'string'", "Unsupported binary op");
    // Verify type mismatch in comparison
    assert::fail("1 < 'a'", "Type mismatch or unsortable types");
}
