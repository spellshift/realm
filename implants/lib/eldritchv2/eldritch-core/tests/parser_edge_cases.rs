use eldritch_core::{ExprKind, Lexer, Parser, StmtKind};

fn parse_full(code: &str) -> (Vec<eldritch_core::Stmt>, Vec<String>) {
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.scan_tokens();
    let mut parser = Parser::new(tokens);
    let (stmts, errors) = parser.parse();
    (stmts, errors.into_iter().map(|e| e.message).collect())
}

#[test]
fn test_annotated_assignment_no_value() {
    let (_stmts, errors) = parse_full("x: int");
    assert!(
        !errors.is_empty(),
        "Expected error for unassigned annotated variable"
    );
    assert!(
        errors[0].contains("Annotated variable must be assigned a value"),
        "Error was: {}",
        errors[0]
    );
}

#[test]
fn test_lambda_annotation_fail() {
    // "lambda x: int: x"
    let (stmts, errors) = parse_full("f = lambda x: int: x");
    assert!(
        !errors.is_empty(),
        "Should have syntax error for trailing tokens"
    );
    assert_eq!(
        stmts.len(),
        1,
        "Should have parsed the first assignment successfully"
    );
}

#[test]
fn test_list_recovery() {
    // "[1, def, 2]"
    let (stmts, errors) = parse_full("x = [1, def, 2]");
    assert!(!errors.is_empty());
    assert_eq!(stmts.len(), 1);

    if let StmtKind::Assignment(_, _, val) = &stmts[0].kind {
        if let ExprKind::List(elements) = &val.kind {
            assert_eq!(elements.len(), 3);
            match &elements[1].kind {
                ExprKind::Error(_) => {}
                _ => panic!("Expected Error expr at index 1, got {:?}", elements[1].kind),
            }
        } else {
            panic!("Expected List");
        }
    } else {
        panic!("Expected Assignment");
    }
}

#[test]
fn test_stmt_synchronization() {
    // "def bad( * )" fails because * must be followed by identifier.
    // It should sync and find x = 1.
    let (stmts, errors) = parse_full("def bad( * )\nx = 1");
    assert!(!errors.is_empty());

    let found_x = stmts.iter().any(|stmt| {
        if let StmtKind::Assignment(lhs, _, _) = &stmt.kind {
            if let ExprKind::Identifier(name) = &lhs.kind {
                return name == "x";
            }
        }
        false
    });

    if !found_x {
        panic!("Did not find assignment to x. Statements: {:#?}", stmts);
    }
}

#[test]
fn test_augmented_assignment_invalid_target() {
    // "1 += 2"
    let (_stmts, errors) = parse_full("1 += 2");
    assert!(
        !errors.is_empty(),
        "Expected error for invalid assignment target"
    );
    assert!(
        errors[0].contains("Invalid assignment target"),
        "Got errors: {:?}",
        errors
    );
}

#[test]
fn test_augmented_assignment_unpacking_fail() {
    // "(a, b) += (1, 2)" - not supported
    let (_stmts, errors) = parse_full("(a, b) += (1, 2)");
    assert!(!errors.is_empty());
    assert!(errors[0].contains("does not support tuple unpacking"));
}
