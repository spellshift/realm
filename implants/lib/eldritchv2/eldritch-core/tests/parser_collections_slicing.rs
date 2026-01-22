use eldritch_core::{ExprKind, Lexer, Parser, StmtKind, Value};

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

fn get_expr_from_stmt(stmt: &eldritch_core::Stmt) -> &eldritch_core::ExprKind {
    match &stmt.kind {
        StmtKind::Expression(expr) => &expr.kind,
        StmtKind::Assignment(_, _, expr) => &expr.kind,
        _ => panic!("Expected Expression or Assignment statement"),
    }
}

#[test]
fn test_list_trailing_comma() {
    let code = "[1, 2,]";
    let stmts = parse_stmts(code).expect("Failed to parse list with trailing comma");
    let expr = get_expr_from_stmt(&stmts[0]);
    if let ExprKind::List(elements) = expr {
        assert_eq!(elements.len(), 2);
        if let ExprKind::Literal(Value::Int(val)) = &elements[1].kind {
            assert_eq!(*val, 2);
        } else {
            panic!("Expected Int(2)");
        }
    } else {
        panic!("Expected List");
    }

    // Single element list with trailing comma
    let code = "[1,]";
    let stmts = parse_stmts(code).expect("Failed to parse single element list with trailing comma");
    let expr = get_expr_from_stmt(&stmts[0]);
    if let ExprKind::List(elements) = expr {
        assert_eq!(elements.len(), 1);
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_set_trailing_comma() {
    let code = "{1, 2,}";
    let stmts = parse_stmts(code).expect("Failed to parse set with trailing comma");
    let expr = get_expr_from_stmt(&stmts[0]);
    if let ExprKind::Set(elements) = expr {
        assert_eq!(elements.len(), 2);
    } else {
        panic!("Expected Set");
    }

    // Single element set with trailing comma
    let code = "{1,}";
    let stmts = parse_stmts(code).expect("Failed to parse single element set with trailing comma");
    let expr = get_expr_from_stmt(&stmts[0]);
    if let ExprKind::Set(elements) = expr {
        assert_eq!(elements.len(), 1);
    } else {
        panic!("Expected Set");
    }
}

#[test]
fn test_dict_trailing_comma() {
    let code = "{\"a\": 1, \"b\": 2,}";
    let stmts = parse_stmts(code).expect("Failed to parse dict with trailing comma");
    let expr = get_expr_from_stmt(&stmts[0]);
    if let ExprKind::Dictionary(entries) = expr {
        assert_eq!(entries.len(), 2);
    } else {
        panic!("Expected Dictionary");
    }

    // Single entry dict with trailing comma
    let code = "{\"a\": 1,}";
    let stmts = parse_stmts(code).expect("Failed to parse single entry dict with trailing comma");
    let expr = get_expr_from_stmt(&stmts[0]);
    if let ExprKind::Dictionary(entries) = expr {
        assert_eq!(entries.len(), 1);
    } else {
        panic!("Expected Dictionary");
    }
}

#[test]
fn test_tuple_trailing_comma() {
    // 1 element tuple must have comma or it's just parens
    let code = "(1,)";
    let stmts = parse_stmts(code).expect("Failed to parse tuple with trailing comma");
    let expr = get_expr_from_stmt(&stmts[0]);
    if let ExprKind::Tuple(elements) = expr {
        assert_eq!(elements.len(), 1);
    } else {
        panic!("Expected Tuple, got {:?}", expr);
    }

    // 2 elements with trailing comma
    let code = "(1, 2,)";
    let stmts = parse_stmts(code).expect("Failed to parse tuple(2) with trailing comma");
    let expr = get_expr_from_stmt(&stmts[0]);
    if let ExprKind::Tuple(elements) = expr {
        assert_eq!(elements.len(), 2);
    } else {
        panic!("Expected Tuple");
    }
}

#[test]
fn test_nested_collections_trailing_commas() {
    let code = "[{1,}, (2,), {\"a\": 1,}]";
    let stmts = parse_stmts(code).expect("Failed to parse nested collections");
    let expr = get_expr_from_stmt(&stmts[0]);
    if let ExprKind::List(elements) = expr {
        assert_eq!(elements.len(), 3);

        // Check Set
        if let ExprKind::Set(set_elems) = &elements[0].kind {
            assert_eq!(set_elems.len(), 1);
        } else {
            panic!("Expected Set at index 0");
        }

        // Check Tuple
        if let ExprKind::Tuple(tup_elems) = &elements[1].kind {
            assert_eq!(tup_elems.len(), 1);
        } else {
            panic!("Expected Tuple at index 1");
        }

        // Check Dict
        if let ExprKind::Dictionary(dict_entries) = &elements[2].kind {
            assert_eq!(dict_entries.len(), 1);
        } else {
            panic!("Expected Dictionary at index 2");
        }
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_slicing_ast_structure() {
    // x[1:2:3]
    let code = "x[1:2:3]";
    let stmts = parse_stmts(code).expect("Failed to parse full slice");
    let expr = get_expr_from_stmt(&stmts[0]);
    if let ExprKind::Slice(target, start, stop, step) = expr {
        if let ExprKind::Identifier(name) = &target.kind {
            assert_eq!(name, "x");
        } else {
            panic!("Expected Identifier target");
        }
        assert!(start.is_some());
        assert!(stop.is_some());
        assert!(step.is_some());
    } else {
        panic!("Expected Slice");
    }

    // x[:]
    let code = "x[:]";
    let stmts = parse_stmts(code).expect("Failed to parse simple slice");
    let expr = get_expr_from_stmt(&stmts[0]);
    if let ExprKind::Slice(_, start, stop, step) = expr {
        assert!(start.is_none());
        assert!(stop.is_none());
        assert!(step.is_none());
    } else {
        panic!("Expected Slice for x[:]");
    }

    // x[::]
    let code = "x[::]";
    let stmts = parse_stmts(code).expect("Failed to parse step slice empty");
    let expr = get_expr_from_stmt(&stmts[0]);
    if let ExprKind::Slice(_, start, stop, step) = expr {
        assert!(start.is_none());
        assert!(stop.is_none());
        assert!(step.is_none()); // step is None if empty in syntax? Logic might imply it.
    } else {
        panic!("Expected Slice for x[::]");
    }

    // x[1:]
    let code = "x[1:]";
    let stmts = parse_stmts(code).expect("Failed to parse start slice");
    let expr = get_expr_from_stmt(&stmts[0]);
    if let ExprKind::Slice(_, start, stop, step) = expr {
        assert!(start.is_some());
        assert!(stop.is_none());
        assert!(step.is_none());
    } else {
        panic!("Expected Slice for x[1:]");
    }

    // x[:2]
    let code = "x[:2]";
    let stmts = parse_stmts(code).expect("Failed to parse stop slice");
    let expr = get_expr_from_stmt(&stmts[0]);
    if let ExprKind::Slice(_, start, stop, step) = expr {
        assert!(start.is_none());
        assert!(stop.is_some());
        assert!(step.is_none());
    } else {
        panic!("Expected Slice for x[:2]");
    }

    // x[::2]
    let code = "x[::2]";
    let stmts = parse_stmts(code).expect("Failed to parse step slice");
    let expr = get_expr_from_stmt(&stmts[0]);
    if let ExprKind::Slice(_, start, stop, step) = expr {
        assert!(start.is_none());
        assert!(stop.is_none());
        assert!(step.is_some());
    } else {
        panic!("Expected Slice for x[::2]");
    }
}
