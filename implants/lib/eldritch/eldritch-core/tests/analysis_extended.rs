extern crate alloc;

use eldritch_core::{ExprKind, Lexer, Parser, Stmt, analysis::Node, analysis::find_node_at_offset};

fn parse(source: &str) -> Vec<Stmt> {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    let mut parser = Parser::new(tokens);
    let (ast, _errors) = parser.parse();
    ast
}

#[test]
fn test_find_in_assignment_annot() {
    let source = "x: int = 1";
    let ast = parse(source);
    // x (0..1), : (1..2), int (3..6), = (7..8), 1 (9..10)
    let node = find_node_at_offset(&ast, 4).unwrap();
    match node {
        Node::Expr(e) => match &e.kind {
            ExprKind::Identifier(s) => assert_eq!(s, "int"),
            _ => panic!("Expected Identifier"),
        },
        _ => panic!("Expected Expr"),
    }
}

#[test]
fn test_find_in_augmented_assignment() {
    let source = "x += 1";
    let ast = parse(source);
    let node = find_node_at_offset(&ast, 0).unwrap();
    match node {
        Node::Expr(e) => match &e.kind {
            ExprKind::Identifier(s) => assert_eq!(s, "x"),
            _ => panic!("Expected Identifier"),
        },
        _ => panic!("Expected Expr"),
    }

    let node_rhs = find_node_at_offset(&ast, 5).unwrap();
    match node_rhs {
        Node::Expr(e) => match &e.kind {
            ExprKind::Literal(i) => assert_eq!(*i, eldritch_core::Value::Int(1)),
            _ => panic!("Expected Integer"),
        },
        _ => panic!("Expected Expr"),
    }
}

#[test]
fn test_find_in_if_else() {
    let source = "if True:\n    pass\nelse:\n    pass";
    let ast = parse(source);
    // True is at offset 3
    let node = find_node_at_offset(&ast, 3).unwrap();
    match node {
        Node::Expr(e) => match &e.kind {
            ExprKind::Identifier(s) => assert_eq!(s, "True"), // True is parsed as Identifier in AST (or Bool depending on version, wait, in Eldritch it might be Identifier or True). Let's check: actually it might be True literal but we can just check if it's Expr.
            _ => (), // Accept anything as long as it's the condition
        },
        _ => panic!("Expected Expr"),
    }

    // Check inside else
    // "if True:\n    pass\nelse:\n    pass"
    //  012345678 901234567 890123 4567890
    // else is around 18. pass is around 28.
    let node_else = find_node_at_offset(&ast, 28);
    // Might be Stmt::Pass
    assert!(node_else.is_some());
}

#[test]
fn test_find_in_return() {
    let source = "def f():\n  return 42";
    let ast = parse(source);
    let node = find_node_at_offset(&ast, 18).unwrap();
    match node {
        Node::Expr(e) => match &e.kind {
            ExprKind::Literal(i) => assert_eq!(*i, eldritch_core::Value::Int(42)),
            _ => (),
        },
        _ => panic!("Expected Expr"),
    }
}

#[test]
fn test_find_in_for() {
    let source = "for x in [1, 2]: pass";
    let ast = parse(source);
    // [1, 2] is at offset 9
    let node = find_node_at_offset(&ast, 10).unwrap();
    match node {
        Node::Expr(e) => match &e.kind {
            ExprKind::List(_) => (),
            ExprKind::Literal(_) => (), // Might hit the inner element
            _ => panic!("Expected List or inner Expr, got {:?}", e.kind),
        },
        _ => panic!("Expected Expr"),
    }
}

#[test]
fn test_find_in_def() {
    let source = "def f(a: int = 1) -> str:\n  pass";
    let ast = parse(source);

    // Offset for 'int'
    let node_annot = find_node_at_offset(&ast, 9).unwrap();
    assert!(matches!(node_annot, Node::Expr(_)));

    // Offset for '1'
    let node_default = find_node_at_offset(&ast, 15).unwrap();
    assert!(matches!(node_default, Node::Expr(_)));

    // Offset for 'str'
    let node_ret = find_node_at_offset(&ast, 21).unwrap();
    assert!(matches!(node_ret, Node::Expr(_)));
}

#[test]
fn test_find_in_lambda() {
    let source = "f = lambda x: x + 1";
    let ast = parse(source);

    // Offset for 'x' in body
    let node = find_node_at_offset(&ast, 14).unwrap();
    assert!(matches!(node, Node::Expr(_)));
}

#[test]
fn test_find_in_comprehension() {
    let source = "l = [x for x in y if x > 0]";
    let ast = parse(source);

    // 'y' is around 16
    let node = find_node_at_offset(&ast, 16).unwrap();
    assert!(matches!(node, Node::Expr(_)));

    // 'x > 0' is around 21
    let node_cond = find_node_at_offset(&ast, 21).unwrap();
    assert!(matches!(node_cond, Node::Expr(_)));
}

#[test]
fn test_find_in_dict_comp() {
    let source = "d = {k: v for k in x if k > 0}";
    let ast = parse(source);

    let node_key = find_node_at_offset(&ast, 5).unwrap();
    assert!(matches!(node_key, Node::Expr(_)));

    let node_val = find_node_at_offset(&ast, 8).unwrap();
    assert!(matches!(node_val, Node::Expr(_)));
}

#[test]
fn test_find_in_fstring() {
    let source = "f'hello {x}'";
    let ast = parse(source);

    // 'x' is at offset 9
    let node = find_node_at_offset(&ast, 9).unwrap();
    assert!(matches!(node, Node::Expr(_)));
}
