use eldritch_core::{Lexer, TokenKind};

fn scan_kinds(source: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(source.to_string());
    lexer.scan_tokens().into_iter().map(|t| t.kind).collect()
}

#[test]
fn test_identifiers_and_keywords() {
    let source = "def foo if return True False None";
    let tokens = scan_kinds(source);
    let expected = vec![
        TokenKind::Def,
        TokenKind::Identifier("foo".to_string()),
        TokenKind::If,
        TokenKind::Return,
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
    let source = "123 123.45 .45";
    let tokens = scan_kinds(source);
    let expected = vec![
        TokenKind::Integer(123),
        TokenKind::Float(123.45),
        TokenKind::Float(0.45),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_strings_simple() {
    let source = r#"'single' "double""#;
    let tokens = scan_kinds(source);
    let expected = vec![
        TokenKind::String("single".to_string()),
        TokenKind::String("double".to_string()),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_strings_triple_single() {
    let source = r#"'''triple'''"#;
    let tokens = scan_kinds(source);
    let expected = vec![
        TokenKind::String("triple".to_string()),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_strings_triple_double() {
    let source = r#""""triple_double""""#;
    let tokens = scan_kinds(source);
    let expected = vec![
        TokenKind::String("triple_double".to_string()),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_escapes() {
    let source = r#""\n\t\"""#;
    let tokens = scan_kinds(source);
    let expected = vec![
        TokenKind::String("\n\t\"".to_string()),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_prefixes() {
    let source = r#"r"raw\n" b"bytes""#;
    let tokens = scan_kinds(source);

    // Check raw string and bytes
    assert_eq!(tokens[0], TokenKind::String("raw\\n".to_string()));
    // Bytes tokens contain Vec<u8>
    if let TokenKind::Bytes(b) = &tokens[1] {
        assert_eq!(b, &b"bytes".to_vec());
    } else {
        panic!("Expected Bytes token, got {:?}", tokens[1]);
    }
}

#[test]
fn test_fstrings() {
    let source = r#"f"val: {x + 1}""#;
    let tokens = scan_kinds(source);

    // Verify FStringContent structure
    // Expected: FStringContent containing [String("val: "), LParen, Identifier("x"), Plus, Integer(1), RParen]
    if let TokenKind::FStringContent(inner_tokens) = &tokens[0] {
        let inner_kinds: Vec<TokenKind> = inner_tokens.iter().map(|t| t.kind.clone()).collect();
        let expected_inner = vec![
            TokenKind::String("val: ".to_string()),
            TokenKind::LParen,
            TokenKind::Identifier("x".to_string()),
            TokenKind::Plus,
            TokenKind::Integer(1),
            TokenKind::RParen,
        ];
        assert_eq!(inner_kinds, expected_inner);
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0]);
    }
}

#[test]
fn test_operators_and_punctuation() {
    // Note: **= parses as StarStar then Assign
    // //= parses as SlashSlashAssign
    let source = "+ - * / % ** // += -= *= /= %= //= **=";
    let tokens = scan_kinds(source);
    let expected = vec![
        TokenKind::Plus,
        TokenKind::Minus,
        TokenKind::Star,
        TokenKind::Slash,
        TokenKind::Percent,
        TokenKind::StarStar,
        TokenKind::SlashSlash,
        TokenKind::PlusAssign,
        TokenKind::MinusAssign,
        TokenKind::StarAssign,
        TokenKind::SlashAssign,
        TokenKind::PercentAssign,
        TokenKind::SlashSlashAssign,
        TokenKind::StarStar,
        TokenKind::Assign,
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_indentation() {
    let source = "
def foo:
  pass
  pass
";
    // Leading newline generates Newline token
    // Indent of 2 spaces
    // Dedent at EOF
    let tokens = scan_kinds(source);

    let expected = vec![
        TokenKind::Newline,
        TokenKind::Def,
        TokenKind::Identifier("foo".to_string()),
        TokenKind::Colon,
        TokenKind::Newline,
        TokenKind::Indent,
        TokenKind::Pass,
        TokenKind::Newline,
        TokenKind::Pass,
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_comments_and_newlines() {
    let source = "# comment\nx # comment\n";
    let tokens = scan_kinds(source);

    // Empty lines (including those from comments) are often collapsed by indentation logic.
    // "# comment\n" -> Newline
    // "x" -> Identifier("x")
    // "# comment\n" -> Newline
    // Implicit final newline from source string + Lexer implicit newline = "\n\n"
    // The indentation logic collapses multiple newlines when indentation doesn't change.
    // So we expect: Newline, Identifier("x"), Newline, Eof.

    let expected = vec![
        TokenKind::Newline,
        TokenKind::Identifier("x".to_string()),
        TokenKind::Newline,
        TokenKind::Eof,
    ];

    assert_eq!(tokens, expected);
}
