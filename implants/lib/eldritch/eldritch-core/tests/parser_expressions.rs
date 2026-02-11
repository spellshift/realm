use eldritch_core::{ExprKind, Lexer, Parser, Stmt, StmtKind, TokenKind};

fn parse_stmts(code: &str) -> Result<Vec<Stmt>, String> {
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

// Helper to inspect the first expression of the first statement
fn get_first_expr(stmts: Vec<Stmt>) -> ExprKind {
    assert_eq!(stmts.len(), 1);
    match &stmts[0].kind {
        StmtKind::Expression(expr) => expr.kind.clone(),
        StmtKind::Assignment(_, _, expr) => expr.kind.clone(), // For cases like `x = ...`
        _ => panic!("Expected expression or assignment statement"),
    }
}

#[test]
fn test_bitwise_precedence() {
    // 1 | 2 & 3 ^ 4
    // Expected: 1 | (2 & (3 ^ 4)) if we follow C/Rust/Python precedence generally?
    // Python Precedence (highest to lowest):
    // ~
    // *, /, //, %
    // +, -
    // <<, >>
    // &
    // ^
    // |

    // Test: 1 | 2 ^ 3 & 4
    // Should parse as: 1 | (2 ^ (3 & 4))

    let code = "1 | 2 ^ 3 & 4";
    let stmts = parse_stmts(code).expect("Failed to parse bitwise");
    let expr = get_first_expr(stmts);

    // Top level should be |
    if let ExprKind::BinaryOp(left, op, right) = expr {
        assert!(matches!(op, TokenKind::BitOr));
        // left: 1
        if let ExprKind::Literal(_val) = left.kind {
            // 1
        } else {
            panic!("Expected 1");
        }

        // right: 2 ^ (3 & 4)
        if let ExprKind::BinaryOp(_r_left, r_op, r_right) = right.kind {
            assert!(matches!(r_op, TokenKind::BitXor));
            // r_left: 2

            // r_right: 3 & 4
            if let ExprKind::BinaryOp(_, rr_op, _) = r_right.kind {
                assert!(matches!(rr_op, TokenKind::BitAnd));
            } else {
                panic!("Expected & inside ^");
            }
        } else {
            panic!("Expected ^ inside |");
        }
    } else {
        panic!("Expected BitOr at top level");
    }
}

#[test]
fn test_shift_precedence() {
    // 1 << 2 + 3
    // + is higher than <<
    // Should be: 1 << (2 + 3)
    let code = "1 << 2 + 3";
    let stmts = parse_stmts(code).expect("Failed to parse shift");
    let expr = get_first_expr(stmts);

    if let ExprKind::BinaryOp(_, op, right) = expr {
        assert!(matches!(op, TokenKind::LShift));
        // right should be 2 + 3 (BinaryOp)
        if let ExprKind::BinaryOp(_, r_op, _) = right.kind {
            assert!(matches!(r_op, TokenKind::Plus));
        } else {
            panic!("Expected + inside <<");
        }
    } else {
        panic!("Expected LShift at top level");
    }
}

#[test]
fn test_ternary_nested() {
    // a if b else c if d else e
    // Should be: a if b else (c if d else e) (right associative)
    let code = "1 if 2 else 3 if 4 else 5";
    let stmts = parse_stmts(code).expect("Failed to parse ternary");
    let expr = get_first_expr(stmts);

    if let ExprKind::If {
        cond: _,
        then_branch: _,
        else_branch,
    } = expr
    {
        // cond: 2, then: 1
        // else_branch: 3 if 4 else 5
        if let ExprKind::If {
            cond: _c2,
            then_branch: _t2,
            else_branch: _e2,
        } = else_branch.kind
        {
            // c2: 4, t2: 3, e2: 5
        } else {
            panic!("Expected nested If in else branch");
        }
    } else {
        panic!("Expected If expression");
    }
}

#[test]
fn test_slicing_complex() {
    // x[1:2:3]
    let code = "x[1:2:3]";
    let stmts = parse_stmts(code).expect("Failed to parse slice");
    let expr = get_first_expr(stmts);

    if let ExprKind::Slice(_, start, stop, step) = expr {
        assert!(start.is_some());
        assert!(stop.is_some());
        assert!(step.is_some());
    } else {
        panic!("Expected Slice");
    }

    // x[::-1]
    let code = "x[::-1]";
    let stmts = parse_stmts(code).expect("Failed to parse negative step slice");
    let expr = get_first_expr(stmts);
    if let ExprKind::Slice(_, start, stop, step) = expr {
        assert!(start.is_none());
        assert!(stop.is_none());
        assert!(step.is_some());
    } else {
        panic!("Expected Slice with step only");
    }
}

#[test]
fn test_lambda_args_kwargs() {
    // lambda *args, **kwargs: 1
    let code = "lambda *args, **kwargs: 1";
    let stmts = parse_stmts(code).expect("Failed to parse lambda");
    let expr = get_first_expr(stmts);

    if let ExprKind::Lambda { params, body: _ } = expr {
        assert_eq!(params.len(), 2);
        match &params[0] {
            eldritch_core::Param::Star(n, _) => assert_eq!(n, "args"),
            _ => panic!("Expected *args"),
        }
        match &params[1] {
            eldritch_core::Param::StarStar(n, _) => assert_eq!(n, "kwargs"),
            _ => panic!("Expected **kwargs"),
        }
    } else {
        panic!("Expected Lambda");
    }
}

#[test]
fn test_comprehensions_nested() {
    // [[x for x in y] for y in z]
    let code = "[[x for x in y] for y in z]";
    let stmts = parse_stmts(code).expect("Failed to parse nested comp");
    let expr = get_first_expr(stmts);

    if let ExprKind::ListComp {
        body,
        vars,
        iterable: _,
        ..
    } = expr
    {
        assert_eq!(vars[0], "y");
        // body is another list comp
        if let ExprKind::ListComp {
            vars: inner_vars, ..
        } = body.kind
        {
            assert_eq!(inner_vars[0], "x");
        } else {
            panic!("Expected nested ListComp");
        }
    } else {
        panic!("Expected ListComp");
    }
}

#[test]
fn test_error_handling() {
    // Missing closing brace
    let code = "{ 'a': 1 ";
    let err = parse_stmts(code).expect_err("Should fail");
    assert!(err.contains("Expected '}'") || err.contains("Expected ','"));

    // Missing closing bracket
    let code = "[1, 2";
    let err = parse_stmts(code).expect_err("Should fail");
    assert!(err.contains("Expected ']'") || err.contains("Expected ','"));

    // Invalid lambda syntax
    let _code = "lambda: 1"; // missing params before :? No, lambda with no params is valid?
    // lambda: 1 is valid in python? Yes.
    // Let's test invalid lambda
    let code = "lambda 1: 1"; // 1 is not a param name
    let err = parse_stmts(code).expect_err("Should fail");
    // "Expected parameter name" or similar
    assert!(err.contains("Expected parameter name") || err.contains("Expected ':'"));
}
