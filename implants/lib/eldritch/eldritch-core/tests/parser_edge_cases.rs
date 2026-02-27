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
fn test_complex_func_def_mixed_args() {
    // def foo(a, b=1, *args: list, **kwargs: dict) -> None: pass
    let code = "def foo(a, b=1, *args: list, **kwargs: dict) -> None: pass";
    let stmts = parse_stmts(code).expect("Failed to parse complex function def");

    assert_eq!(stmts.len(), 1);
    match &stmts[0].kind {
        StmtKind::Def(name, params, ret, _) => {
            assert_eq!(name, "foo");
            assert_eq!(params.len(), 4);

            // Check a
            match &params[0] {
                Param::Normal(n, anno) => {
                    assert_eq!(n, "a");
                    assert!(anno.is_none());
                }
                _ => panic!("Expected Normal param a"),
            }

            // Check b=1
            match &params[1] {
                Param::WithDefault(n, anno, _) => {
                    assert_eq!(n, "b");
                    assert!(anno.is_none());
                }
                _ => panic!("Expected WithDefault param b"),
            }

            // Check *args: list
            match &params[2] {
                Param::Star(n, anno) => {
                    assert_eq!(n, "args");
                    assert!(anno.is_some());
                }
                _ => panic!("Expected Star param args"),
            }

            // Check **kwargs: dict
            match &params[3] {
                Param::StarStar(n, anno) => {
                    assert_eq!(n, "kwargs");
                    assert!(anno.is_some());
                }
                _ => panic!("Expected StarStar param kwargs"),
            }

            assert!(ret.is_some());
        }
        _ => panic!("Expected Def statement"),
    }
}

#[test]
fn test_invalid_func_def_args() {
    // Non-default argument follows default argument
    let code = "def foo(a=1, b): pass";
    let err = parse_stmts(code).expect_err("Should fail");
    assert!(err.contains("Non-default argument follows default argument"));

    // Duplicate argument name (parser might not catch this, but let's check basic syntax errors)
    // Actually the parser structure allows duplicate names in AST, semantic check would catch it.
    // Let's test syntax error: missing name after *
    let code = "def foo(*): pass";
    let err = parse_stmts(code).expect_err("Should fail");
    assert!(err.contains("Expected name after *."));
}

#[test]
fn test_nested_comprehensions() {
    // [x for x in [y for y in range(5)]]
    let code = "l = [x for x in [y for y in range(5)]]";
    let stmts = parse_stmts(code).expect("Failed to parse nested list comp");

    if let StmtKind::Assignment(_, _, expr) = &stmts[0].kind {
        if let ExprKind::ListComp { iterable, .. } = &expr.kind {
            // The iterable should be another ListComp
            if let ExprKind::ListComp { .. } = &iterable.kind {
                // Good
            } else {
                panic!("Expected nested ListComp");
            }
        } else {
            panic!("Expected ListComp");
        }
    }
}

#[test]
fn test_dict_comp_nested() {
    // {k: [v for v in range(3)] for k in range(2)}
    let code = "d = {k: [v for v in range(3)] for k in range(2)}";
    let stmts = parse_stmts(code).expect("Failed to parse nested dict comp");

    if let StmtKind::Assignment(_, _, expr) = &stmts[0].kind {
        if let ExprKind::DictComp { value, .. } = &expr.kind {
            if let ExprKind::ListComp { .. } = &value.kind {
                // Good
            } else {
                panic!("Expected nested ListComp in DictComp value");
            }
        } else {
            panic!("Expected DictComp");
        }
    }
}

#[test]
fn test_complex_fstring() {
    // f"Result: {x + 1} and {y}"
    let code = "s = f\"Result: {x + 1} and {y}\"";
    let stmts = parse_stmts(code).expect("Failed to parse complex f-string");

    if let StmtKind::Assignment(_, _, expr) = &stmts[0].kind {
        if let ExprKind::FString(segments) = &expr.kind {
            assert_eq!(segments.len(), 4); // "Result: ", {x+1}, " and ", {y}
            match &segments[0] {
                FStringSegment::Literal(s) => assert_eq!(s, "Result: "),
                _ => panic!("Expected Literal"),
            }
            match &segments[1] {
                FStringSegment::Expression(_) => {}
                _ => panic!("Expected Expression"),
            }
            match &segments[2] {
                FStringSegment::Literal(s) => assert_eq!(s, " and "),
                _ => panic!("Expected Literal"),
            }
            match &segments[3] {
                FStringSegment::Expression(_) => {}
                _ => panic!("Expected Expression"),
            }
        }
    }
}

#[test]
fn test_nested_fstring() {
    // f"Nested: {f'{x}'}"
    let code = "s = f\"Nested: {f'{x}'}\"";
    let stmts = parse_stmts(code).expect("Failed to parse nested f-string");

    if let StmtKind::Assignment(_, _, expr) = &stmts[0].kind {
        if let ExprKind::FString(segments) = &expr.kind {
            // "Nested: ", {f'{x}'}
            match &segments[1] {
                FStringSegment::Expression(inner) => {
                    if let ExprKind::FString(_) = &inner.kind {
                        // Good
                    } else {
                        panic!("Expected nested FString");
                    }
                }
                _ => panic!("Expected Expression"),
            }
        }
    }
}

#[test]
fn test_lambda_expressions() {
    // lambda: None
    let code = "f = lambda: None";
    let stmts = parse_stmts(code).expect("Failed to parse no-arg lambda");

    if let StmtKind::Assignment(_, _, expr) = &stmts[0].kind {
        if let ExprKind::Lambda { params, .. } = &expr.kind {
            assert_eq!(params.len(), 0);
        } else {
            panic!("Expected Lambda");
        }
    }

    // lambda x, y=1: x+y
    let code = "f = lambda x, y=1: x+y";
    let stmts = parse_stmts(code).expect("Failed to parse complex lambda");
    if let StmtKind::Assignment(_, _, expr) = &stmts[0].kind {
        if let ExprKind::Lambda { params, .. } = &expr.kind {
            assert_eq!(params.len(), 2);
            match &params[1] {
                Param::WithDefault(n, _, _) => assert_eq!(n, "y"),
                _ => panic!("Expected default param"),
            }
        } else {
            panic!("Expected Lambda");
        }
    }
}

#[test]
fn test_slice_syntax() {
    // l[::-1]
    let code = "x = l[::-1]";
    let stmts = parse_stmts(code).expect("Failed to parse step slice");

    if let StmtKind::Assignment(_, _, expr) = &stmts[0].kind {
        if let ExprKind::Slice(_, start, stop, step) = &expr.kind {
            assert!(start.is_none());
            assert!(stop.is_none());
            assert!(step.is_some());
        } else {
            panic!("Expected Slice");
        }
    }

    // l[1:2]
    let code = "x = l[1:2]";
    let stmts = parse_stmts(code).expect("Failed to parse range slice");
    if let StmtKind::Assignment(_, _, expr) = &stmts[0].kind {
        if let ExprKind::Slice(_, start, stop, step) = &expr.kind {
            assert!(start.is_some());
            assert!(stop.is_some());
            assert!(step.is_none());
        } else {
            panic!("Expected Slice");
        }
    }

    // l[1:]
    let code = "x = l[1:]";
    let stmts = parse_stmts(code).expect("Failed to parse start-only slice");
    if let StmtKind::Assignment(_, _, expr) = &stmts[0].kind {
        if let ExprKind::Slice(_, start, stop, step) = &expr.kind {
            assert!(start.is_some());
            assert!(stop.is_none());
            assert!(step.is_none());
        } else {
            panic!("Expected Slice");
        }
    }
}
