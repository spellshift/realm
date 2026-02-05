use eldritch_core::{Lexer, TokenKind};

fn lex(source: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(String::from(source));
    lexer.scan_tokens().into_iter().map(|t| t.kind).collect()
}

#[test]
fn test_inconsistent_indentation() {
    // 2 spaces then 1 space
    let input = "if True:\n  pass\n pass";
    let tokens = lex(input);
    // The error token might not be the very last one due to how the loop works (it might scan pass/newline/eof after error),
    // but it should be present.
    let error_found = tokens.iter().any(|t| match t {
        TokenKind::Error(msg) => msg.contains("Inconsistent indentation"),
        _ => false,
    });
    assert!(
        error_found,
        "Expected inconsistent indentation error, tokens: {:?}",
        tokens
    );
}

#[test]
fn test_unterminated_string_escape_eof() {
    let input = r#""\"#; // String starting with " then backslash then EOF
    let tokens = lex(input);
    match &tokens[0] {
        TokenKind::Error(msg) => assert!(msg.contains("Unterminated string literal")),
        _ => panic!("Expected Error token, got {:?}", tokens[0]),
    }
}

#[test]
fn test_fstring_unmatched_brace() {
    // Must use triple quotes so newline doesn't terminate the string before we see the inner error
    let input = "f\"\"\"{\n\"\"\"";
    let tokens = lex(input);
    // The result should be FStringContent containing the Error
    match &tokens[0] {
        TokenKind::FStringContent(inner) => {
            let error_found = inner.iter().any(|t| match &t.kind {
                TokenKind::Error(msg) => msg.contains("Unmatched '{'"),
                _ => false,
            });
            assert!(
                error_found,
                "Expected unmatched brace error in f-string content: {:?}",
                inner
            );
        }
        _ => panic!("Expected FStringContent, got {:?}", tokens[0]),
    }
}

#[test]
fn test_raw_string_escapes() {
    // r"\"" should be valid string containing \ and "
    let input = r#"r"\"""#;
    let tokens = lex(input);
    let expected = vec![
        TokenKind::String(String::from("\\\"")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);

    // r"\\" should be valid string containing \\
    let input2 = r#"r"\\""#;
    let tokens2 = lex(input2);
    let expected2 = vec![
        TokenKind::String(String::from("\\\\")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens2, expected2);
}

#[test]
fn test_bitwise_not() {
    let input = "~x";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::BitNot,
        TokenKind::Identifier(String::from("x")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}
