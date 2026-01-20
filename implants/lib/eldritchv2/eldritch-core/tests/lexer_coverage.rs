use eldritch_core::{Lexer, TokenKind};

fn tokenize(source: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(source.to_string());
    lexer
        .scan_tokens()
        .into_iter()
        .map(|t| t.kind)
        .collect()
}

#[test]
fn test_identifiers_and_keywords() {
    let source = "def var_name if else True False None";
    let tokens = tokenize(source);

    let expected = vec![
        TokenKind::Def,
        TokenKind::Identifier("var_name".to_string()),
        TokenKind::If,
        TokenKind::Else,
        TokenKind::True,
        TokenKind::False,
        TokenKind::None,
        TokenKind::Newline,
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_numbers() {
    let source = "123 45.67 .89 0";
    let tokens = tokenize(source);

    let expected = vec![
        TokenKind::Integer(123),
        TokenKind::Float(45.67),
        TokenKind::Float(0.89),
        TokenKind::Integer(0),
        TokenKind::Newline,
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_strings_basic() {
    let source = "'single' \"double\"";
    let tokens = tokenize(source);

    let expected = vec![
        TokenKind::String("single".to_string()),
        TokenKind::String("double".to_string()),
        TokenKind::Newline,
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_strings_triple() {
    let source = "'''triple ' quote''' \"\"\"triple \" quote\"\"\"";
    let tokens = tokenize(source);

    let expected = vec![
        TokenKind::String("triple ' quote".to_string()),
        TokenKind::String("triple \" quote".to_string()),
        TokenKind::Newline,
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_strings_escaped() {
    let source = "'escaped \\n \\t \\\\ \\''";
    let tokens = tokenize(source);

    let expected = vec![
        TokenKind::String("escaped \n \t \\ '".to_string()),
        TokenKind::Newline,
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_bytes_literal() {
    let source = "b'bytes' b\"bytes\"";
    let tokens = tokenize(source);

    let expected = vec![
        TokenKind::Bytes(b"bytes".to_vec()),
        TokenKind::Bytes(b"bytes".to_vec()),
        TokenKind::Newline,
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_raw_strings() {
    let source = "r'raw \\n' r\"raw \\t\"";
    let tokens = tokenize(source);

    let expected = vec![
        TokenKind::String("raw \\n".to_string()),
        TokenKind::String("raw \\t".to_string()),
        TokenKind::Newline,
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_f_strings_basic() {
    let source = "f'hello {name}'";
    let tokens = tokenize(source);

    // Based on Lexer implementation, f-strings are tokenized into parts
    // but the final token returned for the whole thing might be FStringContent
    // containing sub-tokens.

    match &tokens[0] {
        TokenKind::FStringContent(parts) => {
            // parts should contain: String("hello "), LParen, Identifier("name"), RParen
            assert_eq!(parts.len(), 4);
            assert_eq!(parts[0].kind, TokenKind::String("hello ".to_string()));
            assert_eq!(parts[1].kind, TokenKind::LParen);
            assert_eq!(parts[2].kind, TokenKind::Identifier("name".to_string()));
            assert_eq!(parts[3].kind, TokenKind::RParen);
        },
        _ => panic!("Expected FStringContent, got {:?}", tokens[0]),
    }
}

#[test]
fn test_operators() {
    let source = "+ - * / % ** // & | ^ ~ << >>";
    let tokens = tokenize(source);

    let expected = vec![
        TokenKind::Plus,
        TokenKind::Minus,
        TokenKind::Star,
        TokenKind::Slash,
        TokenKind::Percent,
        TokenKind::StarStar,
        TokenKind::SlashSlash,
        TokenKind::BitAnd,
        TokenKind::BitOr,
        TokenKind::BitXor,
        TokenKind::BitNot,
        TokenKind::LShift,
        TokenKind::RShift,
        TokenKind::Newline,
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_augmented_assignments() {
    let source = "+= -= *= /= %= //= =";
    let tokens = tokenize(source);

    let expected = vec![
        TokenKind::PlusAssign,
        TokenKind::MinusAssign,
        TokenKind::StarAssign,
        TokenKind::SlashAssign,
        TokenKind::PercentAssign,
        TokenKind::SlashSlashAssign,
        TokenKind::Assign,
        TokenKind::Newline,
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_comparisons() {
    let source = "== != < > <= >= ->";
    let tokens = tokenize(source);

    let expected = vec![
        TokenKind::Eq,
        TokenKind::NotEq,
        TokenKind::Lt,
        TokenKind::Gt,
        TokenKind::LtEq,
        TokenKind::GtEq,
        TokenKind::Arrow,
        TokenKind::Newline,
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_delimiters() {
    let source = "( ) [ ] { } , : . ;";
    let tokens = tokenize(source);

    let expected = vec![
        TokenKind::LParen,
        TokenKind::RParen,
        TokenKind::LBracket,
        TokenKind::RBracket,
        TokenKind::LBrace,
        TokenKind::RBrace,
        TokenKind::Comma,
        TokenKind::Colon,
        TokenKind::Dot,
        TokenKind::Newline, // ; is treated as newline
        TokenKind::Newline, // Implicit newline
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_indentation_simple() {
    let source = "if True:\n  pass";
    let tokens = tokenize(source);

    let expected = vec![
        TokenKind::If,
        TokenKind::True,
        TokenKind::Colon,
        TokenKind::Newline,
        TokenKind::Indent,
        TokenKind::Pass,
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_indentation_nested() {
    let source = "if True:\n  if False:\n    pass";
    let tokens = tokenize(source);

    let expected = vec![
        TokenKind::If,
        TokenKind::True,
        TokenKind::Colon,
        TokenKind::Newline,
        TokenKind::Indent,
        TokenKind::If,
        TokenKind::False,
        TokenKind::Colon,
        TokenKind::Newline,
        TokenKind::Indent,
        TokenKind::Pass,
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Dedent,
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_comments() {
    let source = "# This is a comment\nx = 1 # Inline comment";
    let tokens = tokenize(source);

    let expected = vec![
        TokenKind::Newline, // Comment line ends with newline
        TokenKind::Identifier("x".to_string()),
        TokenKind::Assign,
        TokenKind::Integer(1),
        TokenKind::Newline, // Implicit newline
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_unterminated_string() {
    let source = "'unterminated";
    let tokens = tokenize(source);

    match &tokens[0] {
        TokenKind::Error(msg) => assert!(msg.contains("Unterminated string literal")),
        _ => panic!("Expected Error, got {:?}", tokens[0]),
    }
}

#[test]
fn test_unknown_character() {
    let source = "?";
    let tokens = tokenize(source);

    match &tokens[0] {
        TokenKind::Error(msg) => assert!(msg.contains("Unexpected character")),
        _ => panic!("Expected Error, got {:?}", tokens[0]),
    }
}

#[test]
fn test_raw_string_escaped_quote() {
    // In raw strings, \' should preserve the backslash but treat ' as literal
    // but the Lexer implementation says:
    // "Escape the quote, but keep the backslash in the string"
    let source = r"r'foo\'bar'";
    let tokens = tokenize(source);

    let expected = vec![
        TokenKind::String("foo\\'bar".to_string()),
        TokenKind::Newline,
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_f_string_nested_braces() {
    let source = "f'{ {1} }'";
    let tokens = tokenize(source);

    match &tokens[0] {
        TokenKind::FStringContent(parts) => {
            // Should parse { {1} } as an expression -> set/dict literal {1}
            // Parts: LParen, LBrace, Integer(1), RBrace, RParen
            // Note: The lexer recursively calls itself for the expression
            assert!(parts.len() >= 3);
            assert_eq!(parts[0].kind, TokenKind::LParen);
            assert_eq!(parts[1].kind, TokenKind::LBrace);
            assert_eq!(parts[2].kind, TokenKind::Integer(1));
            assert_eq!(parts[3].kind, TokenKind::RBrace);
            assert_eq!(parts[4].kind, TokenKind::RParen);
        },
        _ => panic!("Expected FStringContent, got {:?}", tokens[0]),
    }
}

#[test]
fn test_indentation_error() {
    let source = "if True:\n  pass\n please_fail";
    let tokens = tokenize(source);

    // " please_fail" has 1 space indent, "  pass" has 2.
    // This should probably be an error or dedent depending on context.
    // Wait, 2 spaces -> indent 2.
    // 1 space -> dedent? But 1 space isn't in the stack (0, 2).
    // So it should be an Inconsistent indentation error.

    // Check for error
    let has_error = tokens.iter().any(|t| matches!(t, TokenKind::Error(_)));
    assert!(has_error, "Expected inconsistent indentation error, got {:?}", tokens);
}
