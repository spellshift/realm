use eldritch_core::{Lexer, TokenKind};

// Helper to check token kind without accessing Token struct directly
fn check_kinds(source: &str, expected: Vec<TokenKind>) {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();

    let actual_kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind.clone()).collect();
    assert_eq!(actual_kinds, expected, "Source: {}", source);
}

#[test]
fn test_operators_power_vs_floordiv_assign() {
    // **= -> StarStar, Assign
    check_kinds(
        "**=",
        vec![
            TokenKind::StarStar,
            TokenKind::Assign,
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );

    // //= -> SlashSlashAssign
    check_kinds(
        "//=",
        vec![
            TokenKind::SlashSlashAssign,
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_numeric_literals_edge_cases() {
    // 1. -> Integer(1), Dot
    check_kinds(
        "1.",
        vec![
            TokenKind::Integer(1),
            TokenKind::Dot,
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );

    // 1.0 -> Float(1.0)
    check_kinds(
        "1.0",
        vec![TokenKind::Float(1.0), TokenKind::Newline, TokenKind::Eof],
    );

    // .1 -> Float(0.1)
    check_kinds(
        ".1",
        vec![TokenKind::Float(0.1), TokenKind::Newline, TokenKind::Eof],
    );
}

#[test]
fn test_raw_strings() {
    // r"hello\nworld" -> String("hello\\nworld") (literal backslash)
    check_kinds(
        r#"r"hello\nworld""#,
        vec![
            TokenKind::String("hello\\nworld".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );

    // r"hello\"world" -> String("hello\\\"world")
    check_kinds(
        r#"r"hello\"world""#,
        vec![
            TokenKind::String("hello\\\"world".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_bytes_literals() {
    check_kinds(
        r#"b"hello""#,
        vec![
            TokenKind::Bytes(b"hello".to_vec()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_comments_newline() {
    // Comment should be followed by Newline
    check_kinds("# comment", vec![TokenKind::Newline, TokenKind::Eof]);

    check_kinds(
        "x # comment",
        vec![
            TokenKind::Identifier("x".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_indentation_simple() {
    let source = "def foo():\n  pass";

    check_kinds(
        source,
        vec![
            TokenKind::Def,
            TokenKind::Identifier("foo".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::Colon,
            TokenKind::Newline,
            TokenKind::Indent,
            TokenKind::Pass,
            TokenKind::Newline,
            TokenKind::Dedent,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_fstring_nesting() {
    // f"{x}" -> FStringContent containing [LParen, Identifier(x), RParen]

    let source = r#"f"{x}""#;
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();

    // Check first token is FStringContent
    // The lexer emits [FStringContent, Newline, Eof] for this input
    match &tokens[0].kind {
        TokenKind::FStringContent(inner_tokens) => {
            let inner_kinds: Vec<TokenKind> = inner_tokens.iter().map(|t| t.kind.clone()).collect();
            assert_eq!(
                inner_kinds,
                vec![
                    TokenKind::LParen,
                    TokenKind::Identifier("x".to_string()),
                    TokenKind::RParen
                ]
            );
        }
        _ => panic!("Expected FStringContent, got {:?}", tokens[0].kind),
    }
}
