use eldritch_core::{analysis::Node, find_node_at_offset, ExprKind, Lexer, Parser, Stmt};

fn parse(source: &str) -> Vec<Stmt> {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    let mut parser = Parser::new(tokens);
    let (ast, errors) = parser.parse();
    if !errors.is_empty() {
        // We allow some errors if they are due to incomplete syntax that we are testing
    }
    ast
}

#[test]
fn test_find_node_at_offset_basic() {
    let source = "x = 1";
    let ast = parse(source);
    // x (0..1), = (2..3), 1 (4..5)
    // Offset 0: 'x'
    let node = find_node_at_offset(&ast, 0).unwrap();
    match node {
        Node::Expr(e) => match &e.kind {
            ExprKind::Identifier(s) => assert_eq!(s, "x"),
            _ => panic!("Expected Identifier"),
        },
        _ => panic!("Expected Expr"),
    }
}

#[test]
fn test_find_node_at_offset_dot_incomplete() {
    let source = "sys.";
    let ast = parse(source);
    // sys (0..3), . (3..4). Missing identifier.
    // Span of GetAttr should cover "sys.".
    // Cursor at 4.

    let stmt = &ast[0];
    println!("AST: {:?}", stmt);

    let node = find_node_at_offset(&ast, 4).expect("Should find node at offset 4");

    match node {
        Node::Expr(e) => match &e.kind {
            ExprKind::GetAttr(lhs, name) => {
                match &lhs.kind {
                    ExprKind::Identifier(s) => assert_eq!(s, "sys"),
                    _ => panic!("Expected Identifier on LHS"),
                }
                assert_eq!(name, "");
            }
            _ => panic!("Expected GetAttr, got {:?}", e.kind),
        },
        _ => panic!("Expected Expr, got {:?}", node),
    }
}

#[test]
fn test_find_node_at_offset_dot_nested() {
    let source = "sys.path.";
    let ast = parse(source);
    // sys (0..3) . (3..4) path (4..8) . (8..9)
    // Cursor at 9.

    let node = find_node_at_offset(&ast, 9).expect("Should find node at offset 9");

    match node {
        Node::Expr(e) => match &e.kind {
            ExprKind::GetAttr(lhs, name) => {
                // LHS should be GetAttr(sys, path)
                match &lhs.kind {
                    ExprKind::GetAttr(inner_lhs, inner_name) => {
                         match &inner_lhs.kind {
                            ExprKind::Identifier(s) => assert_eq!(s, "sys"),
                            _ => panic!("Expected Identifier 'sys'"),
                         }
                         assert_eq!(inner_name, "path");
                    }
                    _ => panic!("Expected GetAttr on LHS"),
                }
                assert_eq!(name, "");
            }
            _ => panic!("Expected GetAttr, got {:?}", e.kind),
        },
        _ => panic!("Expected Expr, got {:?}", node),
    }
}

#[test]
fn test_find_node_inside_function() {
    let source = "def foo():\n  x = 1";
    // def foo(): 0..10. newline. x = 1.
    // x is at 13 (assuming \n is 1 char)

    let ast = parse(source);
    let offset = 13; // Position of 'x'
    let node = find_node_at_offset(&ast, offset).expect("Should find node");

    match node {
        Node::Expr(e) => match &e.kind {
            ExprKind::Identifier(s) => assert_eq!(s, "x"),
            _ => panic!("Expected Identifier 'x', got {:?}", e.kind),
        },
        _ => panic!("Expected Expr"),
    }
}
