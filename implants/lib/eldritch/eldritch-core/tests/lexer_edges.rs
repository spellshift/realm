use eldritch_core::{Lexer, TokenKind};

fn lex(source: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(String::from(source));
    lexer.scan_tokens().into_iter().map(|t| t.kind).collect()
}

#[test]
fn test_numbers() {
    let input = "1 1.0 .5 0";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::Integer(1),
        TokenKind::Float(1.0),
        TokenKind::Float(0.5),
        TokenKind::Integer(0),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_number_trailing_dot() {
    // 1. should be Integer(1) followed by Dot
    let input = "1.";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::Integer(1),
        TokenKind::Dot,
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_dot_no_digit() {
    let input = ".a";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::Dot,
        TokenKind::Identifier(String::from("a")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_strings() {
    let input = r#""hello" 'world' b"bytes" r"raw""#;
    let tokens = lex(input);
    let expected = vec![
        TokenKind::String(String::from("hello")),
        TokenKind::String(String::from("world")),
        TokenKind::Bytes(b"bytes".to_vec()),
        TokenKind::String(String::from("raw")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_escape_sequences() {
    let input = r#""\n\t\\\"""#;
    let tokens = lex(input);
    let expected = vec![
        TokenKind::String(String::from("\n\t\\\"")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_fstring() {
    let input = "f\"{x}\"";
    let tokens = lex(input);
    // Expect: FStringContent([LParen, Identifier(x), RParen]), Newline, Eof

    if let TokenKind::FStringContent(inner_tokens) = &tokens[0] {
        let kinds: Vec<TokenKind> = inner_tokens.iter().map(|t| t.kind.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::LParen,
                TokenKind::Identifier(String::from("x")),
                TokenKind::RParen
            ]
        );
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0]);
    }
}

#[test]
fn test_operators() {
    let input = "+= -= == != ->";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::PlusAssign,
        TokenKind::MinusAssign,
        TokenKind::Eq,
        TokenKind::NotEq,
        TokenKind::Arrow,
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_indentation() {
    // "a\n    b"
    let input = "a\n    b";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::Identifier(String::from("a")),
        TokenKind::Newline,
        TokenKind::Indent,
        TokenKind::Identifier(String::from("b")),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}
