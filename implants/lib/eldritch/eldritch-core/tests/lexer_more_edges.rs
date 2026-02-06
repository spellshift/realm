use eldritch_core::{Lexer, TokenKind};

fn lex(source: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(String::from(source));
    lexer.scan_tokens().into_iter().map(|t| t.kind).collect()
}

#[test]
fn test_integer_overflow() {
    // Current behavior: unwrap_or(0) returns 0 on parse failure (overflow)
    let input = "9223372036854775808"; // i64::MAX + 1
    let tokens = lex(input);
    let expected = vec![TokenKind::Integer(0), TokenKind::Newline, TokenKind::Eof];
    assert_eq!(tokens, expected);
}

#[test]
fn test_mixed_indentation() {
    // Current behavior: whitespace characters are counted equally
    let input = "a\n \tb";
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

#[test]
fn test_token_separation() {
    let input = "+ = +=";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::Plus,
        TokenKind::Assign,
        TokenKind::PlusAssign,
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_keyword_lookalikes() {
    let input = "defe iff truer";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::Identifier(String::from("defe")),
        TokenKind::Identifier(String::from("iff")),
        TokenKind::Identifier(String::from("truer")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_nested_fstrings() {
    let input = "f\"a {f'b {1}'} c\"";
    let tokens = lex(input);

    // We expect FStringContent containing:
    // String("a "), LParen, FStringContent(...), RParen, String(" c")

    if let TokenKind::FStringContent(inner_tokens) = &tokens[0] {
        let kinds: Vec<TokenKind> = inner_tokens.iter().map(|t| t.kind.clone()).collect();

        // Check structure roughly
        assert_eq!(kinds.len(), 5);

        match &kinds[0] {
            TokenKind::String(s) => assert_eq!(s, "a "),
            _ => panic!("Expected String"),
        }

        assert_eq!(kinds[1], TokenKind::LParen);

        // Inner f-string
        match &kinds[2] {
            TokenKind::FStringContent(inner_inner) => {
                let inner_kinds: Vec<TokenKind> =
                    inner_inner.iter().map(|t| t.kind.clone()).collect();
                // "b ", LParen, Integer(1), RParen
                assert_eq!(inner_kinds.len(), 4);
                match &inner_kinds[0] {
                    TokenKind::String(s) => assert_eq!(s, "b "),
                    _ => panic!("Expected inner string"),
                }
                assert_eq!(inner_kinds[1], TokenKind::LParen);
                assert_eq!(inner_kinds[2], TokenKind::Integer(1));
                assert_eq!(inner_kinds[3], TokenKind::RParen);
            }
            _ => panic!("Expected nested FStringContent"),
        }

        assert_eq!(kinds[3], TokenKind::RParen);

        match &kinds[4] {
            TokenKind::String(s) => assert_eq!(s, " c"),
            _ => panic!("Expected String"),
        }
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0]);
    }
}

#[test]
fn test_unicode_escapes_unsupported() {
    // Current behavior: \u is not a special escape, so it becomes 'u'
    let input = r#""\u0041""#;
    let tokens = lex(input);
    let expected = vec![
        TokenKind::String(String::from("u0041")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_unterminated_string_eof() {
    let input = "\"abc";
    let tokens = lex(input);
    match &tokens[0] {
        TokenKind::Error(msg) => assert!(msg.contains("Unterminated string literal")),
        _ => panic!("Expected Error token"),
    }
}
