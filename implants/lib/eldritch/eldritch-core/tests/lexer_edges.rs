use eldritch_core::{Lexer, TokenKind};

fn lex(source: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(String::from(source));
    lexer.scan_tokens().into_iter().map(|t| t.kind).collect()
}

#[test]
fn test_numbers_basic() {
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
fn test_numbers_edge_cases() {
    let input = "00 123.";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::Integer(0),
        TokenKind::Integer(123),
        TokenKind::Dot,
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_numbers_hex_unsupported() {
    let input = "0x123";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::Integer(0),
        TokenKind::Identifier(String::from("x123")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_number_trailing_dot() {
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
fn test_strings_variants() {
    let input = r#""double" 'single' b"bytes" r"raw""#;
    let tokens = lex(input);
    let expected = vec![
        TokenKind::String(String::from("double")),
        TokenKind::String(String::from("single")),
        TokenKind::Bytes(b"bytes".to_vec()),
        TokenKind::String(String::from("raw")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_string_space() {
    let input = r#"" ""#; // " "
    let tokens = lex(input);
    let expected = vec![
        TokenKind::String(String::from(" ")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_string_empty_double() {
    let input = r#""""#; // ""
    let tokens = lex(input);
    let expected = vec![
        TokenKind::String(String::from("")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_string_empty_single() {
    let input = "''";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::String(String::from("")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_strings_mixed_empty() {
    // Re-attempting the failing test case from before but split slightly to debug if needed
    // The previous failure was on the second token.
    let input = r#"" " "" ''"#;
    let tokens = lex(input);
    let expected = vec![
        TokenKind::String(String::from(" ")),
        TokenKind::String(String::from("")),
        TokenKind::String(String::from("")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_escape_sequences() {
    let input = r#""\n\t\\\"\'""#;
    let tokens = lex(input);
    let expected = vec![
        TokenKind::String(String::from("\n\t\\\"\'")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_fstring_simple() {
    let input = "f\"{x}\"";
    let tokens = lex(input);
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
fn test_fstring_complex() {
    let input = "f\"val: {a + 1}\"";
    let tokens = lex(input);

    if let TokenKind::FStringContent(inner_tokens) = &tokens[0] {
        let kinds: Vec<TokenKind> = inner_tokens.iter().map(|t| t.kind.clone()).collect();
        let expected = vec![
            TokenKind::String(String::from("val: ")),
            TokenKind::LParen,
            TokenKind::Identifier(String::from("a")),
            TokenKind::Plus,
            TokenKind::Integer(1),
            TokenKind::RParen,
        ];
        assert_eq!(kinds, expected);
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0]);
    }
}

#[test]
fn test_triple_strings() {
    let input = "\"\"\"line1\nline2\"\"\"";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::String(String::from("line1\nline2")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_string_unterminated() {
    let input = "\"hello";
    let tokens = lex(input);
    match &tokens[0] {
        TokenKind::Error(msg) => assert!(msg.contains("Unterminated string")),
        _ => panic!("Expected Error token"),
    }
}

#[test]
fn test_number_trailing_dot_limitation() {
    // Current limitation: "1." is lexed as Integer(1) followed by Dot.
    // In many languages this is a float (1.0).
    // This test asserts the *current* behavior to detect if it changes.
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
fn test_scientific_notation_unsupported() {
    // Current limitation: Scientific notation is not supported.
    // "1e10" is lexed as Integer(1) followed by Identifier(e10).
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
fn test_scientific_notation_float_unsupported() {
    // "1.5e10" -> Float(1.5), Identifier(e10)
    let input = "1.5e10";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::Float(1.5),
        TokenKind::Identifier(String::from("e10")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_string_newline_error() {
    let input = "\"line1\nline2\"";
    let tokens = lex(input);
    match &tokens[0] {
        TokenKind::Error(msg) => assert!(msg.contains("Unterminated string literal (newline)")),
        _ => panic!("Expected Error token"),
    }
}

#[test]
fn test_operators_extended() {
    let input = "+= -= *= /= %= //= **= &= |= ^= <<= >>=";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::PlusAssign,
        TokenKind::MinusAssign,
        TokenKind::StarAssign,
        TokenKind::SlashAssign,
        TokenKind::PercentAssign,
        TokenKind::SlashSlashAssign,
        // **=
        TokenKind::StarStar,
        TokenKind::Assign,
        // &=
        TokenKind::BitAnd,
        TokenKind::Assign,
        // |=
        TokenKind::BitOr,
        TokenKind::Assign,
        // ^=
        TokenKind::BitXor,
        TokenKind::Assign,
        // <<=
        TokenKind::LShift,
        TokenKind::Assign,
        // >>=
        TokenKind::RShift,
        TokenKind::Assign,
        TokenKind::Newline,
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_operators_comparison() {
    let input = "== != < > <= >= ->";
    let tokens = lex(input);
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
fn test_keywords() {
    let input =
        "def if elif else return for in True False None and or not break continue pass lambda";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::Def,
        TokenKind::If,
        TokenKind::Elif,
        TokenKind::Else,
        TokenKind::Return,
        TokenKind::For,
        TokenKind::In,
        TokenKind::True,
        TokenKind::False,
        TokenKind::None,
        TokenKind::And,
        TokenKind::Or,
        TokenKind::Not,
        TokenKind::Break,
        TokenKind::Continue,
        TokenKind::Pass,
        TokenKind::Lambda,
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_delimiters() {
    let input = "() [] {} , :";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::LParen,
        TokenKind::RParen,
        TokenKind::LBracket,
        TokenKind::RBracket,
        TokenKind::LBrace,
        TokenKind::RBrace,
        TokenKind::Comma,
        TokenKind::Colon,
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_comments() {
    let input = "x = 1 # comment\n# comment line\ny = 2";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::Identifier(String::from("x")),
        TokenKind::Assign,
        TokenKind::Integer(1),
        TokenKind::Newline,
        TokenKind::Newline,
        TokenKind::Identifier(String::from("y")),
        TokenKind::Assign,
        TokenKind::Integer(2),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_indentation_simple() {
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

#[test]
fn test_indentation_complex() {
    let input = "a\n    b\n        c\n    d\ne";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::Identifier(String::from("a")),
        TokenKind::Newline,
        TokenKind::Indent,
        TokenKind::Identifier(String::from("b")),
        TokenKind::Newline,
        TokenKind::Indent,
        TokenKind::Identifier(String::from("c")),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Identifier(String::from("d")),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Identifier(String::from("e")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_empty_input() {
    let input = "";
    let tokens = lex(input);
    let expected = vec![TokenKind::Newline, TokenKind::Eof];
    assert_eq!(tokens, expected);
}

#[test]
fn test_whitespace_only() {
    let input = "   \n  ";
    let tokens = lex(input);
    // Indentation logic skips blank lines (lines with only whitespace/comment)
    // So only one Newline is emitted for the whole thing.
    let expected = vec![TokenKind::Newline, TokenKind::Eof];
    assert_eq!(tokens, expected);
}

#[test]
fn test_unknown_char() {
    let input = "?";
    let tokens = lex(input);
    match &tokens[0] {
        TokenKind::Error(msg) => assert!(msg.contains("Unexpected character")),
        _ => panic!("Expected Error token"),
    }
}
