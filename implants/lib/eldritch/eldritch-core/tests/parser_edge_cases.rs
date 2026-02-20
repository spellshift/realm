use eldritch_core::{ExprKind, Lexer, Parser, StmtKind, TokenKind};

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

// Return ExprKind instead of Expr since Expr is not re-exported at top level
fn parse_expr_kind(code: &str) -> Result<ExprKind, String> {
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.scan_tokens();
    let mut parser = Parser::new(tokens);
    let (stmts, errors) = parser.parse();

    if !errors.is_empty() {
        return Err(errors[0].message.clone());
    }
    if stmts.len() != 1 {
        return Err(format!("Expected 1 statement, got {}", stmts.len()));
    }

    match &stmts[0].kind {
        StmtKind::Expression(expr) => Ok(expr.kind.clone()),
        _ => Err("Expected Expression statement".to_string()),
    }
}

#[test]
fn test_chained_comparisons() {
    let code = "a < b < c";
    let kind = parse_expr_kind(code).expect("Failed to parse chained comparison");
    // Should be (a < b) < c
    match kind {
        ExprKind::BinaryOp(left, op1, _) => {
            match left.kind {
                ExprKind::BinaryOp(_, op2, _) => {
                    assert!(matches!(op2, TokenKind::Lt));
                }
                _ => panic!("Expected left to be BinaryOp"),
            }
            assert!(matches!(op1, TokenKind::Lt));
        }
        _ => panic!("Expected BinaryOp"),
    }
}

#[test]
fn test_operator_precedence() {
    let code = "a and b or c";
    let kind = parse_expr_kind(code).expect("Failed to parse logic");
    match kind {
        ExprKind::LogicalOp(left, op, _) => {
            assert!(matches!(op, TokenKind::Or));
            match left.kind {
                ExprKind::LogicalOp(_, op2, _) => {
                    assert!(matches!(op2, TokenKind::And));
                }
                _ => panic!("Expected left to be LogicalOp(And)"),
            }
        }
        _ => panic!("Expected LogicalOp(Or)"),
    }
}

#[test]
fn test_complex_slicing() {
    // x[1:2:3]
    let code = "x[1:2:3]";
    let kind = parse_expr_kind(code).expect("Failed to parse slicing");
    match kind {
        ExprKind::Slice(_, start, stop, step) => {
            assert!(start.is_some());
            assert!(stop.is_some());
            assert!(step.is_some());
        }
        _ => panic!("Expected Slice"),
    }

    // x[::]
    let code = "x[::]";
    let kind = parse_expr_kind(code).expect("Failed to parse empty slicing");
    match kind {
        ExprKind::Slice(_, start, stop, step) => {
            assert!(start.is_none());
            assert!(stop.is_none());
            assert!(step.is_none());
        }
        _ => panic!("Expected Slice"),
    }
}

#[test]
fn test_tuple_indexing() {
    let code = "x[1, 2]";
    let kind = parse_expr_kind(code).expect("Failed to parse tuple indexing");
    match kind {
        ExprKind::Index(_, index) => match index.kind {
            ExprKind::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
            }
            _ => panic!("Expected Tuple index"),
        },
        _ => panic!("Expected Index with Tuple"),
    }
}

#[test]
fn test_lambda_edge_cases() {
    // Lambda with defaults
    // "lambda a, b=1: a+b" parses as an expression.
    // However, it might be ambiguous at stmt level if not parenthesized or assigned?
    // "lambda x: x" is a valid expression statement.
    let code = "lambda a, b=1: a+b";
    let kind = parse_expr_kind(code).expect("Failed to parse lambda with defaults");
    match kind {
        ExprKind::Lambda { params, .. } => {
            assert_eq!(params.len(), 2);
        }
        _ => panic!("Expected Lambda"),
    }

    // Nested lambda
    // (lambda x: x)(1)
    let code = "(lambda x: x)(1)";
    let kind = parse_expr_kind(code).expect("Failed to parse nested lambda call");
    match kind {
        ExprKind::Call(callee, _) => match callee.kind {
            ExprKind::Lambda { .. } => {}
            _ => panic!("Expected Lambda as callee, got {:?}", callee.kind),
        },
        _ => panic!("Expected Call"),
    }
}

#[test]
fn test_invalid_params_order() {
    // Non-default after default
    let code = "def f(a=1, b): pass";
    let err = parse_stmts(code).expect_err("Should fail: non-default after default");
    assert!(err.contains("Non-default argument follows default argument"));

    // Args after **kwargs
    let code = "def f(**kwargs, a): pass";
    let result = parse_stmts(code);

    match result {
        Ok(_) => panic!("Parser incorrectly accepted `def f(**kwargs, a): pass`"),
        Err(e) => assert!(e.contains("Arguments cannot follow **kwargs")),
    }
}

#[test]
fn test_fstring_nested() {
    // f"nested {f'{x}'}"
    let code = "f\"nested {f'{x}'}\"";
    let kind = parse_expr_kind(code).expect("Failed to parse nested f-string");
    match kind {
        ExprKind::FString(segments) => {
            assert_eq!(segments.len(), 2);
        }
        _ => panic!("Expected FString"),
    }
}
