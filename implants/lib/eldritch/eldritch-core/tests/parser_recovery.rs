use eldritch_core::{ExprKind, Interpreter, Lexer, Parser, StmtKind};

#[test]
fn test_valid_parsing() {
    let code = "x = 1 + 2";
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.scan_tokens();
    let mut parser = Parser::new(tokens);
    let (stmts, errors) = parser.parse();

    assert!(errors.is_empty());
    assert_eq!(stmts.len(), 1);

    let mut interp = Interpreter::new();
    let res = interp.interpret(code);
    assert!(res.is_ok());
}

#[test]
fn test_recoverable_statement_error() {
    // syntax error on line 1, valid on line 2
    let code = "x = 1 +\ny = 2";
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.scan_tokens();
    let mut parser = Parser::new(tokens);
    let (stmts, errors) = parser.parse();

    // Should have 1 error for line 1
    assert_eq!(errors.len(), 1);

    // Should produce 2 statements: Error, Assignment
    assert_eq!(stmts.len(), 2);

    match &stmts[0].kind {
        StmtKind::Error(_) => {}
        _ => panic!("Expected Error statement"),
    }

    match &stmts[1].kind {
        StmtKind::Assignment(_, _, _) => {}
        _ => panic!("Expected Assignment statement"),
    }

    // Interpreter should fail
    let mut interp = Interpreter::new();
    let res = interp.interpret(code);
    assert!(res.is_err());
}

#[test]
fn test_recoverable_list_error() {
    // List with garbage in middle
    // "x = [1, 2, if, 4]" -> 'if' is keyword, not expression start.

    let code = "x = [1, 2, if, 4]";
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.scan_tokens();
    let mut parser = Parser::new(tokens);
    let (stmts, errors) = parser.parse();

    // Should have 1 error
    assert!(!errors.is_empty());

    // Should produce 1 statement (Assignment)
    assert_eq!(stmts.len(), 1);

    if let StmtKind::Assignment(_, _, val) = &stmts[0].kind {
        if let ExprKind::List(elements) = &val.kind {
            // [1, 2, Error, 4]
            assert_eq!(elements.len(), 4);
            match &elements[2].kind {
                ExprKind::Error(_) => {}
                _ => panic!("Expected Error expr in list"),
            }
        } else {
            panic!("Expected list assignment");
        }
    } else {
        panic!("Expected assignment stmt");
    }
}
