use eldritch_core::{Lexer, Parser};

fn parse_stmts(code: &str) -> Result<Vec<eldritch_core::Stmt>, String> {
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.scan_tokens();
    let mut parser = Parser::new(tokens);
    let (stmts, errors) = parser.parse();
    if errors.is_empty() {
        Ok(stmts)
    } else {
        Err(errors[0].message.clone())
    }
}

#[test]
fn test_positional_after_keyword() {
    let code = "f(a=1, 2)";
    let result = parse_stmts(code);
    assert!(
        result.is_err(),
        "Should fail with positional argument after keyword argument"
    );
    assert!(
        result
            .unwrap_err()
            .contains("Positional argument follows keyword argument")
    );
}

#[test]
fn test_positional_after_kwargs() {
    // f(**kwargs, 1) is invalid
    let code = "f(**k, 1)";
    let result = parse_stmts(code);
    assert!(
        result.is_err(),
        "Should fail with positional argument after **kwargs"
    );
}

#[test]
fn test_star_args_after_keyword() {
    // f(a=1, *args) is invalid
    let code = "f(a=1, *args)";
    let result = parse_stmts(code);
    assert!(
        result.is_err(),
        "Should fail with *args after keyword argument"
    );
    assert!(
        result
            .unwrap_err()
            .contains("Iterable argument unpacking follows keyword argument")
    );
}

#[test]
fn test_valid_calls() {
    // f(1, *args, a=2, **kwargs) is valid
    let code = "f(1, *args, a=2, **kwargs)";
    assert!(parse_stmts(code).is_ok());

    // f(1, *args, a=2) is valid
    let code = "f(1, *args, a=2)";
    assert!(parse_stmts(code).is_ok());
}
