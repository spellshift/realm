use eldritch_core::{
    ExprKind, FStringSegment, Lexer, Param, Parser, Stmt, StmtKind, TokenKind, Value,
};

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

fn parse_expr(code: &str) -> ExprKind {
    let stmts = parse_stmts(code).expect("Failed to parse expression statement");
    match stmts.into_iter().next().unwrap().kind {
        StmtKind::Expression(expr) => expr.kind,
        StmtKind::Assignment(_, _, expr) => expr.kind, // Handle if test uses assignment
        _ => panic!("Expected Expression or Assignment statement"),
    }
}

#[test]
fn test_precedence_mul_add() {
    // 1 + 2 * 3 -> 1 + (2 * 3)
    let kind = parse_expr("1 + 2 * 3");
    if let ExprKind::BinaryOp(left, op, right) = kind {
        assert_eq!(op, TokenKind::Plus);
        // left: 1
        if let ExprKind::Literal(Value::Int(i)) = left.kind {
            assert_eq!(i, 1);
        } else {
            panic!("Expected 1 on left");
        }
        // right: 2 * 3
        if let ExprKind::BinaryOp(rl, rop, rr) = right.kind {
            assert_eq!(rop, TokenKind::Star);
            if let ExprKind::Literal(Value::Int(i)) = rl.kind {
                assert_eq!(i, 2);
            }
            if let ExprKind::Literal(Value::Int(i)) = rr.kind {
                assert_eq!(i, 3);
            }
        } else {
            panic!("Expected 2 * 3 on right");
        }
    } else {
        panic!("Expected BinaryOp +");
    }
}

#[test]
fn test_precedence_bitwise() {
    // 1 | 2 & 3 -> 1 | (2 & 3) because & binds tighter
    let kind = parse_expr("1 | 2 & 3");
    if let ExprKind::BinaryOp(_, op, right) = kind {
        assert_eq!(op, TokenKind::BitOr);
        if let ExprKind::BinaryOp(_, rop, _) = right.kind {
            assert_eq!(rop, TokenKind::BitAnd);
        } else {
            panic!("Expected BitAnd on right, got {:?}", right.kind);
        }
    } else {
        panic!("Expected BitOr at top level");
    }
}

#[test]
fn test_precedence_shift_add() {
    // 1 << 2 + 3 -> 1 << (2 + 3) because + binds tighter than <<
    let kind = parse_expr("1 << 2 + 3");
    if let ExprKind::BinaryOp(_, op, right) = kind {
        assert_eq!(op, TokenKind::LShift);
        if let ExprKind::BinaryOp(_, rop, _) = right.kind {
            assert_eq!(rop, TokenKind::Plus);
        } else {
            panic!("Expected Plus on right");
        }
    } else {
        panic!("Expected LShift at top level");
    }
}

#[test]
fn test_associativity_sub() {
    // 1 - 2 - 3 -> (1 - 2) - 3 (Left associative)
    let kind = parse_expr("1 - 2 - 3");
    if let ExprKind::BinaryOp(left, op, _) = kind {
        assert_eq!(op, TokenKind::Minus);
        // left should be 1 - 2
        if let ExprKind::BinaryOp(_, lop, _) = left.kind {
            assert_eq!(lop, TokenKind::Minus);
        } else {
            panic!("Expected Minus on left");
        }
    } else {
        panic!("Expected Minus at top level");
    }
}

#[test]
fn test_comparison_chaining_behavior() {
    // a < b < c -> (a < b) < c in Eldritch (Divergence from Python)
    let kind = parse_expr("1 < 2 < 3");
    if let ExprKind::BinaryOp(left, op, _) = kind {
        assert_eq!(op, TokenKind::Lt); // Outer op is < (comparison against 3)

        // Left should be (1 < 2)
        if let ExprKind::BinaryOp(_, lop, _) = left.kind {
            assert_eq!(lop, TokenKind::Lt);
        } else {
            panic!("Expected (1 < 2) on left");
        }
    } else {
        panic!("Expected BinaryOp < at top level");
    }
}

#[test]
fn test_complex_slicing() {
    // a[1:2:3]
    let kind = parse_expr("a[1:2:3]");
    if let ExprKind::Slice(obj, start, stop, step) = kind {
        if let ExprKind::Identifier(name) = obj.kind {
            assert_eq!(name, "a");
        }
        assert!(start.is_some()); // 1
        assert!(stop.is_some()); // 2
        assert!(step.is_some()); // 3
    } else {
        panic!("Expected Slice [1:2:3]");
    }

    // a[::-1]
    let kind = parse_expr("a[::-1]");
    if let ExprKind::Slice(_, start, stop, step) = kind {
        assert!(start.is_none());
        assert!(stop.is_none());
        assert!(step.is_some());
    } else {
        panic!("Expected Slice [::-1]");
    }
}

#[test]
fn test_tuple_indexing() {
    // d[1, 2] -> Index(d, Tuple(1, 2))
    let kind = parse_expr("d[1, 2]");
    if let ExprKind::Index(obj, index) = kind {
        if let ExprKind::Identifier(name) = obj.kind {
            assert_eq!(name, "d");
        }
        if let ExprKind::Tuple(elements) = index.kind {
            assert_eq!(elements.len(), 2);
        } else {
            panic!("Expected Tuple index");
        }
    } else {
        panic!("Expected Index");
    }
}

#[test]
fn test_lambda_star_args() {
    // lambda *args, **kwargs: 0
    // "f = ..." to make it a statement
    let stmts = parse_stmts("f = lambda *args, **kwargs: 0").expect("Failed to parse lambda");
    if let StmtKind::Assignment(_, _, expr) = stmts[0].kind.clone() {
        if let ExprKind::Lambda { params, .. } = expr.kind {
            assert_eq!(params.len(), 2);
            match &params[0] {
                Param::Star(n, _) => assert_eq!(n, "args"),
                _ => panic!("Expected *args"),
            }
            match &params[1] {
                Param::StarStar(n, _) => assert_eq!(n, "kwargs"),
                _ => panic!("Expected **kwargs"),
            }
        } else {
            panic!("Expected Lambda");
        }
    }
}

#[test]
fn test_fstring_nested_dict() {
    // f"{ {1: 2} }" -> FString containing Dict
    // Assignment needed to handle statement
    let stmts = parse_stmts("x = f\"{ {1: 2} }\"").expect("Failed to parse f-string nested");
    if let StmtKind::Assignment(_, _, expr) = stmts[0].kind.clone() {
        if let ExprKind::FString(segments) = expr.kind {
            assert_eq!(segments.len(), 1);
            if let FStringSegment::Expression(inner) = &segments[0] {
                if let ExprKind::Dictionary(entries) = &inner.kind {
                    assert_eq!(entries.len(), 1);
                } else {
                    panic!("Expected Dictionary inside f-string, got {:?}", inner.kind);
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
fn test_power_operator_missing() {
    // 2 ** 3 should currently FAIL or be parsed incorrectly as 2 then syntax error
    // Verifying current limitation
    let res = parse_stmts("x = 2 ** 3");
    assert!(
        res.is_err(),
        "Expected parse error for missing power operator"
    );
}

#[test]
fn test_list_error_recovery() {
    // [1, <error>, 2]
    // Simulate error by using invalid token sequence inside list
    // "1 +" is incomplete expression
    let _res = parse_stmts("l = [1, 1 +, 2]");
    // The parser might recover and produce an Error expr inside the list,
    // OR it might fail the whole statement if recovery isn't robust enough for this case.
    // The implementation shows `self.expression()` returns Err, caught, pushed to `errors`,
    // loops until comma/bracket, pushes `ExprKind::Error`.
    // BUT `parse()` returns (stmts, errors). My `parse_stmts` returns Err if `errors` is not empty.

    let mut lexer = Lexer::new("l = [1, 1 +, 2]".to_string());
    let tokens = lexer.scan_tokens();
    let mut parser = Parser::new(tokens);
    let (stmts, errors) = parser.parse();

    assert!(!errors.is_empty(), "Expected errors");
    assert_eq!(stmts.len(), 1); // Should still produce a statement with Error expr

    if let StmtKind::Assignment(_, _, expr) = &stmts[0].kind {
        if let ExprKind::List(elements) = &expr.kind {
            assert_eq!(elements.len(), 3);
            match elements[1].kind {
                ExprKind::Error(_) => {} // Success: recovered
                _ => panic!("Expected Error expr in list at index 1"),
            }
        } else {
            panic!("Expected List");
        }
    } else {
        panic!("Expected Assignment");
    }
}
