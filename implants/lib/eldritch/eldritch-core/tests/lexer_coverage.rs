use eldritch_core::{Lexer, TokenKind};

fn lex(source: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(String::from(source));
    lexer.scan_tokens().into_iter().map(|t| t.kind).collect()
}

#[test]
fn test_compound_assignments_split() {
    // These operators are not currently tokenized as single tokens.
    // This test verifies they are split into the operator and an assignment.
    let input = "&= |= ^= <<= >>=";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::BitAnd,
        TokenKind::Assign,
        TokenKind::BitOr,
        TokenKind::Assign,
        TokenKind::BitXor,
        TokenKind::Assign,
        TokenKind::LShift,
        TokenKind::Assign,
        TokenKind::RShift,
        TokenKind::Assign,
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_compound_assignments_star_star() {
    // **= is also split
    let input = "**=";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::StarStar,
        TokenKind::Assign,
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_integer_overflow() {
    // Test that numbers overflowing i64 result in 0 (based on unwrap_or(0))
    let input = "9223372036854775808"; // i64::MAX + 1
    let tokens = lex(input);
    let expected = vec![TokenKind::Integer(0), TokenKind::Newline, TokenKind::Eof];
    assert_eq!(tokens, expected);
}

#[test]
fn test_inconsistent_indentation_recovery() {
    // Test that the lexer emits an error but continues tokenizing valid tokens after inconsistent indentation.
    // Input:
    // if True:
    //   pass
    //  pass  <-- Inconsistent indent (1 space vs 2 expected)

    let input = "if True:\n  pass\n pass";
    let tokens = lex(input);

    // Expected behavior:
    // 1. "if True:" -> If, True, Colon, Newline
    // 2. "  pass" -> Indent, Pass, Newline
    // 3. " pass" -> Error("Inconsistent indentation")
    //    Note: The implicit Dedent from 2 -> 1 spaces is currently lost in the lexer implementation
    //    when an error occurs, as the dedents vector is discarded.
    // 4. Recovery: "pass" -> Pass, Newline, Eof

    let expected_start = vec![
        TokenKind::If,
        TokenKind::True,
        TokenKind::Colon,
        TokenKind::Newline,
        TokenKind::Indent,
        TokenKind::Pass,
        TokenKind::Newline,
    ];

    // Check first part
    assert_eq!(tokens[..7], expected_start[..]);

    // Check error
    match &tokens[7] {
        TokenKind::Error(msg) => assert!(msg.contains("Inconsistent indentation")),
        _ => panic!("Expected Error token at index 7, got {:?}", tokens[7]),
    }

    // Check recovery
    // After error, it should tokenize "pass"
    assert_eq!(tokens[8], TokenKind::Pass);
    assert_eq!(tokens[9], TokenKind::Newline);
    assert_eq!(tokens[10], TokenKind::Eof);
}

#[test]
fn test_fstring_nested_error_priority() {
    // Priority: Outer string errors (e.g. unterminated) should be reported
    // even if the inner f-string expression is also malformed.
    // The lexer scans f-string content recursively. If the outer string is not terminated,
    // the lexer prioritizes reporting "Unterminated string" over internal errors.

    let input = "f\"{";
    let tokens = lex(input);
    match &tokens[0] {
        TokenKind::Error(msg) => assert!(msg.contains("Unterminated string")),
        _ => panic!("Expected Unterminated string error, got {:?}", tokens[0]),
    }
}

#[test]
fn test_triple_quoted_string_with_quotes() {
    let input = "\"\"\" ' \" \"\"\"";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::String(String::from(" ' \" ")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_scientific_notation_as_tokens() {
    // 1e10 -> Integer(1), Identifier(e10)
    let input = "1e10";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::Integer(1),
        TokenKind::Identifier(String::from("e10")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_trailing_dot_behavior() {
    // 123. -> Integer(123), Dot
    let input = "123.";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::Integer(123),
        TokenKind::Dot,
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}
