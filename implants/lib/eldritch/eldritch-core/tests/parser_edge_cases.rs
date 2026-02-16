use eldritch_core::{ExprKind, FStringSegment, Lexer, Parser, StmtKind, TokenKind};

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

fn parse_expr(code: &str) -> eldritch_core::Expr {
    let stmts = parse_stmts(code).expect("Failed to parse expression statement");
    match &stmts[0].kind {
        StmtKind::Expression(expr) => expr.clone(),
        StmtKind::Assignment(_, _, expr) => expr.clone(), // Helper for cases like "x = ..."
        _ => panic!("Expected expression or assignment statement"),
    }
}

#[test]
fn test_chained_comparison_associativity() {
    // Eldritch parses chained comparisons (a < b < c) as left-associative ((a < b) < c).
    // This is a known divergence from Python (which does a < b and b < c).
    let code = "a < b < c";
    let expr = parse_expr(code);

    if let ExprKind::BinaryOp(left, op1, right) = expr.kind {
        assert_eq!(op1, TokenKind::Lt);
        // right should be 'c'
        if let ExprKind::Identifier(name) = right.kind {
            assert_eq!(name, "c");
        } else {
            panic!("Expected right operand to be 'c'");
        }

        // left should be (a < b)
        if let ExprKind::BinaryOp(ll, op2, lr) = left.kind {
            assert_eq!(op2, TokenKind::Lt);
            if let ExprKind::Identifier(name) = ll.kind {
                assert_eq!(name, "a");
            } else {
                panic!("Expected left-left operand to be 'a'");
            }
            if let ExprKind::Identifier(name) = lr.kind {
                assert_eq!(name, "b");
            } else {
                panic!("Expected left-right operand to be 'b'");
            }
        } else {
            panic!("Expected left operand to be a binary op (a < b)");
        }
    } else {
        panic!("Expected BinaryOp");
    }
}

#[test]
fn test_operator_precedence() {
    // a + b * c -> a + (b * c)
    let code = "a + b * c";
    let expr = parse_expr(code);

    if let ExprKind::BinaryOp(left, op, right) = expr.kind {
        assert_eq!(op, TokenKind::Plus);
        // left is a
        if let ExprKind::Identifier(name) = left.kind {
            assert_eq!(name, "a");
        } else {
            panic!("Expected left operand to be 'a'");
        }
        // right is (b * c)
        if let ExprKind::BinaryOp(rl, op2, rr) = right.kind {
            assert_eq!(op2, TokenKind::Star);
            if let ExprKind::Identifier(name) = rl.kind {
                assert_eq!(name, "b");
            }
            if let ExprKind::Identifier(name) = rr.kind {
                assert_eq!(name, "c");
            }
        } else {
            panic!("Expected right operand to be (b * c)");
        }
    } else {
        panic!("Expected BinaryOp");
    }

    // a or b and c -> a or (b and c) (and binds tighter than or)
    let code = "a or b and c";
    let expr = parse_expr(code);

    if let ExprKind::LogicalOp(left, op, right) = expr.kind {
        assert_eq!(op, TokenKind::Or);
        if let ExprKind::Identifier(name) = left.kind {
            assert_eq!(name, "a");
        }
        // right is (b and c)
        if let ExprKind::LogicalOp(rl, op2, rr) = right.kind {
            assert_eq!(op2, TokenKind::And);
            if let ExprKind::Identifier(name) = rl.kind {
                assert_eq!(name, "b");
            }
            if let ExprKind::Identifier(name) = rr.kind {
                assert_eq!(name, "c");
            }
        } else {
            panic!("Expected right operand to be (b and c)");
        }
    } else {
        panic!("Expected LogicalOp");
    }

    // not a == b -> not (a == b) (comparison binds tighter than not)
    // Wait, let's verify Python precedence: 'not a == b' is 'not (a == b)'.
    // In Eldritch: logic_not calls equality calls comparison.
    // So logic_not parses 'not ...', consuming 'not', then calls logic_not recursively.
    // If next is not 'not', it calls equality.
    // equality parses comparison.
    // So 'not a == b' -> 'not (a == b)'.
    let code = "not a == b";
    let expr = parse_expr(code);

    if let ExprKind::UnaryOp(op, right) = expr.kind {
        assert_eq!(op, TokenKind::Not);
        // right is (a == b)
        if let ExprKind::BinaryOp(rl, op2, rr) = right.kind {
            assert_eq!(op2, TokenKind::Eq);
            if let ExprKind::Identifier(name) = rl.kind {
                assert_eq!(name, "a");
            }
            if let ExprKind::Identifier(name) = rr.kind {
                assert_eq!(name, "b");
            }
        } else {
            panic!("Expected right operand to be (a == b)");
        }
    } else {
        panic!("Expected UnaryOp");
    }
}

#[test]
fn test_complex_slicing_syntax() {
    // a[::] -> start=None, stop=None, step=None
    let code = "a[::]";
    let expr = parse_expr(code);
    if let ExprKind::Slice(_, start, stop, step) = expr.kind {
        assert!(start.is_none());
        assert!(stop.is_none());
        assert!(step.is_none()); // step defaults to None in AST if parsed as [::] ?
    // Actually, if [::], step is implicitly None or Some(None)?
    // In parser: match_token(Colon) -> step = Some(expression())? No, if it matches Colon, it checks if RBracket follows.
    // if match_token(Colon) && !check(RBracket) { step = Some(...) }
    // So a[::] means second colon matched, but RBracket followed, so step remains None.
    } else {
        panic!("Expected Slice a[::]");
    }

    // a[1::] -> start=1, stop=None, step=None
    let code = "a[1::]";
    let expr = parse_expr(code);
    if let ExprKind::Slice(_, start, stop, step) = expr.kind {
        assert!(start.is_some());
        assert!(stop.is_none());
        assert!(step.is_none());
    } else {
        panic!("Expected Slice a[1::]");
    }

    // a[:2:] -> start=None, stop=2, step=None
    let code = "a[:2:]";
    let expr = parse_expr(code);
    if let ExprKind::Slice(_, start, stop, step) = expr.kind {
        assert!(start.is_none());
        assert!(stop.is_some());
        assert!(step.is_none());
    } else {
        panic!("Expected Slice a[:2:]");
    }

    // a[::3] -> start=None, stop=None, step=3
    let code = "a[::3]";
    let expr = parse_expr(code);
    if let ExprKind::Slice(_, start, stop, step) = expr.kind {
        assert!(start.is_none());
        assert!(stop.is_none());
        assert!(step.is_some());
    } else {
        panic!("Expected Slice a[::3]");
    }
}

#[test]
fn test_tuple_grouping_distinction() {
    // (x) -> x (grouping)
    let code = "(x)";
    let expr = parse_expr(code);
    if let ExprKind::Identifier(name) = expr.kind {
        assert_eq!(name, "x");
    } else {
        panic!("Expected Identifier x, got {:?}", expr.kind);
    }

    // (x,) -> Tuple([x])
    let code = "(x,)";
    let expr = parse_expr(code);
    if let ExprKind::Tuple(elements) = expr.kind {
        assert_eq!(elements.len(), 1);
        if let ExprKind::Identifier(name) = &elements[0].kind {
            assert_eq!(name, "x");
        }
    } else {
        panic!("Expected Tuple, got {:?}", expr.kind);
    }

    // () -> Tuple([])
    let code = "()";
    let expr = parse_expr(code);
    if let ExprKind::Tuple(elements) = expr.kind {
        assert!(elements.is_empty());
    } else {
        panic!("Expected empty Tuple, got {:?}", expr.kind);
    }
}

#[test]
fn test_lambda_precedence() {
    // lambda x: x + 1 -> body is (x + 1)
    let code = "f = lambda x: x + 1";
    let expr = parse_expr(code); // Returns the lambda expr from assignment

    if let ExprKind::Lambda { params, body } = expr.kind {
        assert_eq!(params.len(), 1);
        // Body should be BinaryOp(x + 1)
        if let ExprKind::BinaryOp(left, op, _right) = body.kind {
            assert_eq!(op, TokenKind::Plus);
            if let ExprKind::Identifier(name) = left.kind {
                assert_eq!(name, "x");
            }
        } else {
            panic!("Expected body to be binary op x+1");
        }
    } else {
        panic!("Expected Lambda");
    }
}

#[test]
fn test_deeply_nested_structures() {
    // [[[[...]]]]
    // Reduced depth to prevent stack overflow in debug builds/test runner
    let depth = 20;
    let mut code = String::new();
    for _ in 0..depth {
        code.push('[');
    }
    code.push('1');
    for _ in 0..depth {
        code.push(']');
    }

    let expr = parse_expr(&code);

    // Verify depth by traversing
    let mut current = &expr;
    for _ in 0..depth {
        if let ExprKind::List(elements) = &current.kind {
            assert_eq!(elements.len(), 1);
            current = &elements[0];
        } else {
            panic!("Expected List");
        }
    }

    if let ExprKind::Literal(_val) = &current.kind {
        // Check value if needed
    } else {
        panic!("Expected Literal at bottom");
    }
}

#[test]
fn test_fstring_complex_nesting() {
    // f"nested: {f'{1}'}"
    let code = "x = f\"nested: {f'{1}'}\"";
    let expr = parse_expr(code);

    if let ExprKind::FString(segments) = expr.kind {
        // "nested: " and expression
        assert_eq!(segments.len(), 2);
        match &segments[1] {
            FStringSegment::Expression(inner_expr) => {
                // Inner expr should be FString too
                if let ExprKind::FString(inner_segments) = &inner_expr.kind {
                    assert_eq!(inner_segments.len(), 1);
                    // Check content of inner fstring
                    match &inner_segments[0] {
                        FStringSegment::Expression(val_expr) => {
                            if let ExprKind::Literal(_) = val_expr.kind {
                                // ok
                            } else {
                                panic!("Expected literal inside inner f-string");
                            }
                        }
                        _ => panic!("Expected expression inside inner f-string"),
                    }
                } else {
                    panic!("Expected inner FString");
                }
            }
            _ => panic!("Expected Expression"),
        }
    } else {
        panic!("Expected FString");
    }
}
