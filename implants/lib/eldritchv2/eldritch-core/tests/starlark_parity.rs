use eldritch_core::Lexer;
use eldritch_core::Parser;

fn parse(code: &str) -> Result<(), String> {
    let tokens = Lexer::new(code.to_string()).scan_tokens()?;
    let mut parser = Parser::new(tokens);
    parser.parse().map(|_| ())
}

// Tests from starlark-rust/starlark_syntax/src/syntax/def_tests

#[test]
fn test_pos_after_default() {
    // "def test(x=1, y): pass" -> Error
    let code = "def test(x=1, y): pass";
    assert!(
        parse(code).is_err(),
        "Positional arg after default should be error"
    );
}

#[test]
fn test_positional_only_cannot_be_first() {
    // "def test(/, x): pass" -> Error (not supported syntax)
    let code = "def test(/, x): pass";
    assert!(
        parse(code).is_err(),
        "Positional only syntax / not supported"
    );
}

#[test]
fn test_positional_only_in_standard_dialect_def() {
    // "def test(/, x): pass" -> Error
    let code = "def test(/, x): pass";
    assert!(parse(code).is_err());
}

#[test]
fn test_positional_only_in_standard_dialect_lambda() {
    // "lambda /, x: 17" -> Error
    let code = "lambda /, x: 17";
    assert!(parse(code).is_err());
}

#[test]
fn test_slash_slash() {
    // "def test(x, /, y, /): pass" -> Error
    let code = "def test(x, /, y, /): pass";
    assert!(parse(code).is_err());
}

#[test]
fn test_star_cannot_be_last() {
    // "def test(x, *): pass" -> Error
    let code = "def test(x, *): pass";
    assert!(parse(code).is_err());
}

#[test]
fn test_star_star() {
    // "def test(*, *): pass" -> Error
    let code = "def test(*, *): pass";
    assert!(parse(code).is_err());
}

#[test]
fn test_star_then_args() {
    // "def test(x, *, *args): pass" -> Error
    // Eldritch doesn't support bare `*` at all, so this fails.
    let code = "def test(x, *, *args): pass";
    assert!(parse(code).is_err());
}

#[test]
fn test_star_then_kwargs() {
    // "def test(x, *, **kwargs): pass" -> Error
    let code = "def test(x, *, **kwargs): pass";
    assert!(parse(code).is_err());
}

// Tests from starlark-rust/starlark_syntax/src/syntax/grammar_tests

#[test]
fn test_assignment_type_annotation() {
    // "(x, y): int = foo" -> Error
    let code = "(x, y): int = foo";
    assert!(parse(code).is_err());
}

#[test]
fn test_bad_assignment_or() {
    // "[x or y] = 1" -> Error
    let code = "[x or y] = 1";
    assert!(
        parse(code).is_err(),
        "Assignment to expression [x or y] should fail"
    );
}

#[test]
fn test_bad_assignment_augmented() {
    // "[x] += 1" -> Error
    let code = "[x] += 1";
    assert!(parse(code).is_err());
}

#[test]
fn test_ellipsis() {
    // "x = ..." -> Error (not supported)
    let code = "x = ...";
    assert!(parse(code).is_err());
}

#[test]
fn test_lambda() {
    // "x = lambda y: y + 1" -> Success (Eldritch supports lambda)
    let code = "x = lambda y: y + 1";
    assert!(parse(code).is_ok());
}

#[test]
fn test_list_in_index_expr() {
    // "x[1, 2] = 3" -> Failure
    let code = "x[1, 2] = 3";
    assert!(parse(code).is_err());
}

#[test]
fn test_top_level_def() {
    // "def toto(): pass" -> Success
    let code = "def toto(): pass";
    assert!(parse(code).is_ok());
}

#[test]
fn test_top_level_statements() {
    // "if x == 1: ..." -> Success
    let code = "x = 1\nif x == 1:\n  x = 2";
    assert!(parse(code).is_ok());
}
