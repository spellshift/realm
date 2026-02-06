use eldritch_core::{Lexer, TokenKind};

fn lex(source: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(String::from(source));
    lexer.scan_tokens().into_iter().map(|t| t.kind).collect()
}

#[test]
fn test_fstring_nested_dict() {
    let input = "f\"{ {'a': 1} }\"";
    let tokens = lex(input);

    if let TokenKind::FStringContent(inner) = &tokens[0] {
        let kinds: Vec<TokenKind> = inner.iter().map(|t| t.kind.clone()).collect();
        let expected = vec![
            TokenKind::LParen,
            TokenKind::LBrace,
            TokenKind::String(String::from("a")),
            TokenKind::Colon,
            TokenKind::Integer(1),
            TokenKind::RBrace,
            TokenKind::RParen,
        ];
        assert_eq!(kinds, expected);
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0]);
    }
}

#[test]
fn test_fstring_recursive() {
    let input = "f\"{ f'{x}' }\"";
    let tokens = lex(input);

    if let TokenKind::FStringContent(inner) = &tokens[0] {
        let kinds: Vec<TokenKind> = inner.iter().map(|t| t.kind.clone()).collect();
        // Inner: LParen, FStringContent(...), RParen
        if let TokenKind::FStringContent(nested_inner) = &kinds[1] {
            let nested_kinds: Vec<TokenKind> =
                nested_inner.iter().map(|t| t.kind.clone()).collect();
            assert_eq!(
                nested_kinds,
                vec![
                    TokenKind::LParen,
                    TokenKind::Identifier(String::from("x")),
                    TokenKind::RParen
                ]
            );
        } else {
            panic!("Expected nested FStringContent, got {:?}", kinds[1]);
        }
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0]);
    }
}

#[test]
fn test_fstring_complex_expr() {
    let input = "f\"{x + y * 2}\"";
    let tokens = lex(input);

    if let TokenKind::FStringContent(inner) = &tokens[0] {
        let kinds: Vec<TokenKind> = inner.iter().map(|t| t.kind.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::LParen,
                TokenKind::Identifier(String::from("x")),
                TokenKind::Plus,
                TokenKind::Identifier(String::from("y")),
                TokenKind::Star,
                TokenKind::Integer(2),
                TokenKind::RParen,
            ]
        );
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0]);
    }
}

#[test]
fn test_fstring_newline_error() {
    let input = "f\"{ \n }\"";
    let tokens = lex(input);

    // The inner expression parser returns an Error token when it hits newline with nesting > 0
    // But `string()` continues...
    // Wait, if inner returns Error, `string()` appends it to `fstring_tokens`.
    // Then `string()` sees `\n` (since `tokenize_fstring_expression` stopped there).
    // `string()` handles `\n` -> "Unterminated string literal (newline)".
    // So we probably get an Error token from `string()` method, effectively masking the inner error?
    // Or does it return `TokenKind::FStringContent` containing the Error token?

    // Let's inspect what we get.
    // If it returns Error, fine.
    // If it returns FStringContent, we check inside.

    match &tokens[0] {
        TokenKind::Error(msg) => {
            // Likely "Unterminated string literal (newline)" because the newline was reached in `string()` loop
            assert!(
                msg.contains("Unterminated string") || msg.contains("Unmatched"),
                "Got: {}",
                msg
            );
        }
        TokenKind::FStringContent(inner) => {
            // Maybe it managed to wrap it?
            let kinds: Vec<TokenKind> = inner.iter().map(|t| t.kind.clone()).collect();
            println!("Inner tokens: {:?}", kinds);
            // Verify one of them is error
            assert!(kinds.iter().any(|k| matches!(k, TokenKind::Error(_))));
        }
        _ => panic!(
            "Expected Error or FStringContent with Error, got {:?}",
            tokens[0]
        ),
    }
}

#[test]
fn test_fstring_unmatched_brace_internal() {
    let input = "f\"{ { }\"";
    let tokens = lex(input);

    // This typically results in "Unterminated string" because the quote is consumed by the inner expression parser as part of the expression
    match &tokens[0] {
        TokenKind::Error(msg) => assert!(
            msg.contains("Unterminated string") || msg.contains("Unmatched"),
            "Got: {}",
            msg
        ),
        _ => panic!("Expected Error, got {:?}", tokens[0]),
    }
}
