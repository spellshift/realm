use eldritch_core::{Expr, ExprKind, FStringSegment, Lexer, Param, Parser, StmtKind, Value};

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

fn parse_expr(code: &str) -> Expr {
    let stmts = parse_stmts(code).unwrap();
    // Assuming expression statement
    if let StmtKind::Expression(expr) = &stmts[0].kind {
        expr.clone()
    } else {
        panic!("Expected expression statement");
    }
}

#[test]
fn test_lambda_variations() {
    // Basic lambda
    let code = "lambda x: x";
    let expr = parse_expr(code);
    if let ExprKind::Lambda { params, body: _ } = expr.kind {
        assert_eq!(params.len(), 1);
        match &params[0] {
            Param::Normal(name, _) => assert_eq!(name, "x"),
            _ => panic!("Expected normal param x"),
        }
    } else {
        panic!("Expected Lambda");
    }

    // Lambda with multiple args and default
    let code = "lambda x, y=2: x + y";
    let expr = parse_expr(code);
    if let ExprKind::Lambda { params, .. } = expr.kind {
        assert_eq!(params.len(), 2);
        match &params[1] {
            Param::WithDefault(name, _, _) => assert_eq!(name, "y"),
            _ => panic!("Expected default param y"),
        }
    } else {
        panic!("Expected Lambda");
    }

    // Lambda with *args
    let code = "lambda *args: args";
    let expr = parse_expr(code);
    if let ExprKind::Lambda { params, .. } = expr.kind {
        assert_eq!(params.len(), 1);
        match &params[0] {
            Param::Star(name, _) => assert_eq!(name, "args"),
            _ => panic!("Expected Star param args"),
        }
    } else {
        panic!("Expected Lambda");
    }
}

#[test]
fn test_lambda_invalid_syntax() {
    // Lambda with annotation (should fail or parse oddly, but checking 'invalid args')
    // As analyzed, "lambda x: int: x" fails because ':' terminates params, and 'int: x' is invalid statement structure in this context if not part of larger expression.
    // If we parse it as "lambda x: int", it returns int.
    // Let's try to parse "lambda x: int: x" as a statement.
    let code = "lambda x: int: x";
    // This expects a list of statements.
    // 1. lambda x: int
    // 2. : x -> Invalid
    let res = parse_stmts(code);
    assert!(res.is_err() || res.unwrap().len() > 1); // Or it might just error on ":"
}

#[test]
fn test_fstring_edge_cases() {
    // Nested braces in expression
    let code = "f\"{ {1, 2} }\"";
    let expr = parse_expr(code);
    if let ExprKind::FString(segments) = expr.kind {
        assert_eq!(segments.len(), 1);
        match &segments[0] {
            FStringSegment::Expression(e) => {
                if let ExprKind::Set(el) = &e.kind {
                    assert_eq!(el.len(), 2);
                } else {
                    panic!("Expected Set expression");
                }
            }
            _ => panic!("Expected Expression segment"),
        }
    } else {
        panic!("Expected FString");
    }
}

#[test]
fn test_comprehensions_extended() {
    // Set comprehension
    let code = "{x for x in range(3)}";
    let expr = parse_expr(code);
    if let ExprKind::SetComp {
        body: _,
        vars,
        iterable: _,
        cond,
    } = expr.kind
    {
        assert_eq!(vars[0], "x");
        assert!(cond.is_none());
    } else {
        panic!("Expected SetComp");
    }

    // Dict comprehension
    let code = "{k:v for k,v in items}";
    let expr = parse_expr(code);
    if let ExprKind::DictComp {
        key: _,
        value: _,
        vars,
        ..
    } = expr.kind
    {
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0], "k");
        assert_eq!(vars[1], "v");
    } else {
        panic!("Expected DictComp");
    }
}

#[test]
fn test_slicing_extended() {
    // Full slice
    let code = "a[1:2:3]";
    let expr = parse_expr(code);
    if let ExprKind::Slice(_obj, start, stop, step) = expr.kind {
        assert!(start.is_some());
        assert!(stop.is_some());
        assert!(step.is_some());
    } else {
        panic!("Expected Slice, got {:?}", expr.kind);
    }

    // Step only
    let code = "a[:: -1]";
    let expr = parse_expr(code);
    if let ExprKind::Slice(_obj, start, stop, step) = expr.kind {
        assert!(start.is_none());
        assert!(stop.is_none());
        assert!(step.is_some());
    } else {
        panic!("Expected Slice");
    }
}

#[test]
fn test_precedence_extended() {
    // 1 + 2 * 3 -> 1 + (2 * 3)
    let code = "1 + 2 * 3";
    let expr = parse_expr(code);
    if let ExprKind::BinaryOp(left, _, right) = expr.kind {
        // op should be +
        // left should be 1
        // right should be 2 * 3
        match left.kind {
            ExprKind::Literal(Value::Int(1)) => {}
            _ => panic!("Left should be 1"),
        }
        // op is Plus (TokenKind)
        // ... I need to check TokenKind match. TokenKind doesn't implement Eq with simple enum matching easily without import.
        // But I can check if right is BinaryOp.
        if let ExprKind::BinaryOp(rl, _, _) = right.kind {
            // rop is Star
            match rl.kind {
                ExprKind::Literal(Value::Int(2)) => {}
                _ => panic!("Right-Left should be 2"),
            }
        } else {
            panic!("Right should be BinaryOp (2*3)");
        }
    } else {
        panic!("Expected BinaryOp");
    }
}

#[test]
fn test_recovery_in_list() {
    // [1, <error>, 2]
    // The parser should recover and produce an Error expr.
    let code = "[1, , 2]"; // Double comma is error
    let res = parse_stmts(code);

    // parse_stmts checks if errors.is_empty(). So it returns Err.
    assert!(res.is_err());

    // But if we use the parser directly we can check the AST even with errors.
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.scan_tokens();
    let mut parser = Parser::new(tokens);
    let (stmts, errors) = parser.parse();
    assert!(!errors.is_empty());
    assert_eq!(stmts.len(), 1);

    if let StmtKind::Expression(expr) = &stmts[0].kind {
        if let ExprKind::List(elements) = &expr.kind {
            assert_eq!(elements.len(), 3); // 1, error, 2
            match &elements[1].kind {
                ExprKind::Error(_) => {}
                _ => panic!("Expected Error expr at index 1"),
            }
        } else {
            panic!("Expected List");
        }
    }
}
