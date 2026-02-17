use eldritch_core::{ExprKind, FStringSegment, Lexer, Parser, StmtKind, TokenKind, Value};

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
fn test_chained_comparisons() {
    let code = "x = 1 < 2 < 3";
    let stmts = parse_stmts(code).expect("Failed to parse chained comparison");

    // Expect: x = (1 < 2) < 3
    if let StmtKind::Assignment(_, _, expr) = &stmts[0].kind {
        if let ExprKind::BinaryOp(left, op1, right) = &expr.kind {
            assert!(matches!(op1, TokenKind::Lt));

            // Right is 3
            if let ExprKind::Literal(Value::Int(v)) = &right.kind {
                assert_eq!(*v, 3);
            } else {
                panic!("Expected literal 3");
            }

            // Left is (1 < 2)
            if let ExprKind::BinaryOp(l_inner, op2, r_inner) = &left.kind {
                assert!(matches!(op2, TokenKind::Lt));
                if let ExprKind::Literal(Value::Int(v)) = &l_inner.kind {
                    assert_eq!(*v, 1);
                }
                if let ExprKind::Literal(Value::Int(v)) = &r_inner.kind {
                    assert_eq!(*v, 2);
                }
            } else {
                panic!("Expected inner binary op");
            }
        } else {
            panic!("Expected outer binary op");
        }
    }
}

#[test]
fn test_slicing_complex() {
    // x[1:2:3]
    let code = "x[1:2:3]"; // expression statement
    let stmts = parse_stmts(code).expect("Failed to parse slice");

    if let StmtKind::Expression(expr) = &stmts[0].kind {
        if let ExprKind::Slice(_target, start, stop, step) = &expr.kind {
            assert!(start.is_some());
            assert!(stop.is_some());
            assert!(step.is_some());
        } else {
            panic!("Expected Slice expression");
        }
    }
}

#[test]
fn test_slicing_tuple_index() {
    // x[1, 2]
    let code = "x[1, 2]";
    let stmts = parse_stmts(code).expect("Failed to parse tuple index");

    if let StmtKind::Expression(expr) = &stmts[0].kind {
        if let ExprKind::Index(_target, index) = &expr.kind {
            if let ExprKind::Tuple(elements) = &index.kind {
                assert_eq!(elements.len(), 2);
            } else {
                panic!("Expected Tuple index");
            }
        } else {
            panic!("Expected Index expression");
        }
    }
}

#[test]
fn test_operator_precedence() {
    // not a == b
    // Should be `not (a == b)`
    let code = "not a == b";
    let stmts = parse_stmts(code).expect("Failed to parse operator precedence");

    if let StmtKind::Expression(expr) = &stmts[0].kind {
        if let ExprKind::UnaryOp(op, operand) = &expr.kind {
            assert!(matches!(op, TokenKind::Not));
            if let ExprKind::BinaryOp(_, op2, _) = &operand.kind {
                assert!(matches!(op2, TokenKind::Eq));
            } else {
                panic!("Expected BinaryOp inside Not");
            }
        } else {
            panic!("Expected UnaryOp (Not)");
        }
    }
}

#[test]
fn test_tuple_grouping() {
    // (1) is 1 (grouping)
    let code = "x = (1)";
    let stmts = parse_stmts(code).expect("Failed to parse grouping");
    if let StmtKind::Assignment(_, _, expr) = &stmts[0].kind {
        // Should be literal 1, not Tuple
        if let ExprKind::Literal(_) = &expr.kind {
            // OK
        } else {
            panic!("Expected Literal, got {:?}", expr.kind);
        }
    }

    // (1,) is Tuple
    let code = "x = (1,)";
    let stmts = parse_stmts(code).expect("Failed to parse tuple literal");
    if let StmtKind::Assignment(_, _, expr) = &stmts[0].kind {
        if let ExprKind::Tuple(elements) = &expr.kind {
            assert_eq!(elements.len(), 1);
        } else {
            panic!("Expected Tuple, got {:?}", expr.kind);
        }
    }
}

#[test]
fn test_fstring_nesting_deep() {
    // f"{f'{x}'}"
    let code = "f\"{f'{x}'}\"";
    let stmts = parse_stmts(code).expect("Failed to parse nested fstring");

    if let StmtKind::Expression(expr) = &stmts[0].kind {
        if let ExprKind::FString(segments) = &expr.kind {
            assert_eq!(segments.len(), 1);
            if let FStringSegment::Expression(inner) = &segments[0] {
                if let ExprKind::FString(inner_segments) = &inner.kind {
                    assert_eq!(inner_segments.len(), 1);
                    if let FStringSegment::Expression(inner_inner) = &inner_segments[0] {
                        if let ExprKind::Identifier(name) = &inner_inner.kind {
                            assert_eq!(name, "x");
                        } else {
                            panic!("Expected Identifier x");
                        }
                    }
                } else {
                    panic!("Expected inner FString");
                }
            } else {
                panic!("Expected Expression segment");
            }
        } else {
            panic!("Expected FString");
        }
    }
}

#[test]
fn test_arg_unpacking_order() {
    // *args before **kwargs
    let code = "def f(*args, **kwargs): pass";
    parse_stmts(code).expect("Valid args order");

    // **kwargs before *args -> Invalid (parser error expected, but verify message)
    let code = "def f(**kwargs, *args): pass";
    parse_stmts(code).expect_err("Invalid args order");
}
