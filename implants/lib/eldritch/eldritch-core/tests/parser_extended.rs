use eldritch_core::{ExprKind, FStringSegment, Lexer, Param, Parser, StmtKind};

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
fn test_func_def_complex_args() {
    // Def with mixed args: pos, default, *args, **kwargs
    let code = "def foo(a, b=1, *args, **kwargs): pass";
    let stmts = parse_stmts(code).expect("Failed to parse complex function def");

    assert_eq!(stmts.len(), 1);
    match &stmts[0].kind {
        StmtKind::Def(name, params, _, _) => {
            assert_eq!(name, "foo");
            assert_eq!(params.len(), 4);

            // Check params
            match &params[0] {
                Param::Normal(n, _) => assert_eq!(n, "a"),
                _ => panic!("Expected Normal param a"),
            }
            match &params[1] {
                Param::WithDefault(n, _, _) => assert_eq!(n, "b"),
                _ => panic!("Expected WithDefault param b"),
            }
            match &params[2] {
                Param::Star(n, _) => assert_eq!(n, "args"),
                _ => panic!("Expected Star param args"),
            }
            match &params[3] {
                Param::StarStar(n, _) => assert_eq!(n, "kwargs"),
                _ => panic!("Expected StarStar param kwargs"),
            }
        }
        _ => panic!("Expected Def statement"),
    }
}

#[test]
fn test_func_def_with_annotations() {
    // Def with annotations
    let code = "def bar(a: int, b: str = 'x') -> bool: pass";
    let stmts = parse_stmts(code).expect("Failed to parse annotated function");
    assert_eq!(stmts.len(), 1);
    match &stmts[0].kind {
        StmtKind::Def(_, params, ret_anno, _) => {
            // Check params annotations presence
            match &params[0] {
                Param::Normal(n, anno) => {
                    assert_eq!(n, "a");
                    assert!(anno.is_some());
                }
                _ => panic!("Expected Normal param a"),
            }
            match &params[1] {
                Param::WithDefault(n, anno, _) => {
                    assert_eq!(n, "b");
                    assert!(anno.is_some());
                }
                _ => panic!("Expected WithDefault param b"),
            }
            // Return annotation
            assert!(ret_anno.is_some());
        }
        _ => panic!("Expected Def statement"),
    }
}

#[test]
fn test_func_def_invalid_args() {
    // Non-default after default
    let code = "def bad(a=1, b): pass";
    let err = parse_stmts(code).expect_err("Should fail");
    assert!(err.contains("Non-default argument follows default argument"));

    // Missing name after *
    let code = "def bad_star(*): pass";
    let err = parse_stmts(code).expect_err("Should fail");
    assert!(err.contains("Expected name after *."));
}

#[test]
fn test_lambda_complex_args() {
    // Lambda with defaults
    let code = "f = lambda a, b=1: a + b";
    let stmts = parse_stmts(code).expect("Failed to parse lambda");
    // This produces an Assignment stmt
    assert_eq!(stmts.len(), 1);

    if let StmtKind::Assignment(_, _, expr) = &stmts[0].kind {
        if let ExprKind::Lambda { params, .. } = &expr.kind {
            assert_eq!(params.len(), 2);
            match &params[1] {
                Param::WithDefault(n, _, _) => assert_eq!(n, "b"),
                _ => panic!("Expected WithDefault param b"),
            }
        } else {
            panic!("Expected Lambda expr");
        }
    } else {
        panic!("Expected Assignment");
    }
}

#[test]
fn test_fstring_parsing() {
    // Basic f-string
    let code = "x = f\"val: {1 + 2}\"";
    let stmts = parse_stmts(code).expect("Failed to parse f-string");

    if let StmtKind::Assignment(_, _, expr) = &stmts[0].kind {
        if let ExprKind::FString(segments) = &expr.kind {
            assert_eq!(segments.len(), 2); // "val: " and expression
            match &segments[0] {
                FStringSegment::Literal(s) => assert_eq!(s, "val: "),
                _ => panic!("Expected Literal"),
            }
            match &segments[1] {
                FStringSegment::Expression(_) => {}
                _ => panic!("Expected Expression"),
            }
        } else {
            panic!("Expected FString expression, got {:?}", expr.kind);
        }
    } else {
        panic!("Expected Assignment");
    }
}

#[test]
fn test_fstring_nested() {
    // Nested f-string: f"nested: {f'{1}'}"
    let code = "x = f\"nested: {f'{1}'}\"";
    let stmts = parse_stmts(code).expect("Failed to parse nested f-string");

    if let StmtKind::Assignment(_, _, expr) = &stmts[0].kind {
        if let ExprKind::FString(segments) = &expr.kind {
            // "nested: " and expression
            assert_eq!(segments.len(), 2);
            match &segments[1] {
                FStringSegment::Expression(inner_expr) => {
                    // Inner expr should be FString too
                    if let ExprKind::FString(inner_segments) = &inner_expr.kind {
                        assert_eq!(inner_segments.len(), 1);
                    } else {
                        panic!("Expected inner FString");
                    }
                }
                _ => panic!("Expected Expression"),
            }
        } else {
            panic!("Expected FString expression");
        }
    }
}

#[test]
fn test_comprehensions_edge_cases() {
    // List comp with complex logic
    let code = "l = [x for x in range(10) if x % 2 == 0]";
    parse_stmts(code).expect("Failed to parse list comp");

    // Dict comp
    let code = "d = {x[0]: x[1] for x in items if x}";
    parse_stmts(code).expect("Failed to parse dict comp");

    // Set comp
    let code = "s = {x for x in range(5)}";
    parse_stmts(code).expect("Failed to parse set comp");
}

#[test]
fn test_invalid_comprehension() {
    // Missing 'in'
    let code = "[x for x range(10)]";
    let err = parse_stmts(code).expect_err("Should fail");
    assert!(err.contains("Expected 'in'"));
}
