use eldritch_core::{Lexer, TokenKind};

#[test]
fn test_numbers() {
    let cases = vec![
        ("123", TokenKind::Integer(123)),
        ("0", TokenKind::Integer(0)),
        ("123.456", TokenKind::Float(123.456)),
        (".5", TokenKind::Float(0.5)),
        ("0.0", TokenKind::Float(0.0)),
    ];

    for (input, expected) in cases {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.scan_tokens();
        // The first token should match.
        // Note: Lexer appends a newline to source, so we might get [Token, Newline, EOF]
        assert_eq!(tokens[0].kind, expected, "Failed for input: {}", input);
    }
}

#[test]
fn test_trailing_dot_number() {
    // "1." is parsed as Integer(1) followed by Dot.
    let input = "1.";
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.scan_tokens();
    assert_eq!(tokens[0].kind, TokenKind::Integer(1));
    assert_eq!(tokens[1].kind, TokenKind::Dot);
}

#[test]
fn test_operators() {
    let cases = vec![
        ("+", TokenKind::Plus),
        ("-", TokenKind::Minus),
        ("*", TokenKind::Star),
        ("/", TokenKind::Slash),
        ("**", TokenKind::StarStar),
        ("//", TokenKind::SlashSlash),
        ("+=", TokenKind::PlusAssign),
        ("-=", TokenKind::MinusAssign),
        ("*=", TokenKind::StarAssign),
        ("/=", TokenKind::SlashAssign),
        ("//=", TokenKind::SlashSlashAssign),
        ("==", TokenKind::Eq),
        ("!=", TokenKind::NotEq),
        ("<=", TokenKind::LtEq),
        (">=", TokenKind::GtEq),
        ("->", TokenKind::Arrow),
        ("&", TokenKind::BitAnd),
        ("|", TokenKind::BitOr),
        ("^", TokenKind::BitXor),
        ("~", TokenKind::BitNot),
        ("<<", TokenKind::LShift),
        (">>", TokenKind::RShift),
    ];

    for (input, expected) in cases {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.scan_tokens();
        assert_eq!(tokens[0].kind, expected, "Failed for input: {}", input);
    }
}

#[test]
fn test_strings() {
    let cases = vec![
        ("\"hello\"", TokenKind::String("hello".to_string())),
        ("'hello'", TokenKind::String("hello".to_string())),
        (
            "r\"raw\\nstring\"",
            TokenKind::String("raw\\nstring".to_string()),
        ),
        ("b\"bytes\"", TokenKind::Bytes(b"bytes".to_vec())),
    ];

    for (input, expected) in cases {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.scan_tokens();
        assert_eq!(tokens[0].kind, expected, "Failed for input: {}", input);
    }
}

#[test]
fn test_fstring_interpolation() {
    let input = "f\"val={x}\"";
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.scan_tokens();

    if let TokenKind::FStringContent(inner_tokens) = &tokens[0].kind {
        // inner_tokens should be:
        // String("val="), LParen, Identifier("x"), RParen
        // The Lexer automatically wraps expressions in parens.
        assert_eq!(inner_tokens.len(), 4);
        assert_eq!(inner_tokens[0].kind, TokenKind::String("val=".to_string()));
        assert_eq!(inner_tokens[1].kind, TokenKind::LParen);
        assert_eq!(inner_tokens[2].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(inner_tokens[3].kind, TokenKind::RParen);
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0].kind);
    }
}

#[test]
fn test_indentation_block() {
    let code = "if True:\n  pass";
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.scan_tokens();

    let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind.clone()).collect();

    // Expected sequence:
    // If, True, Colon, Newline, Indent, Pass, Newline, Dedent, Eof
    // Note: The lexer emits Newline after Colon because of the \n in input.
    // Then it sees 2 spaces, so emits Indent.
    // Then Pass.
    // Then implicitly appends \n -> Newline.
    // Then EOF -> emits Dedent for remaining indent stack.

    let relevant: Vec<TokenKind> = kinds
        .into_iter()
        .filter(|k| {
            matches!(
                k,
                TokenKind::If
                    | TokenKind::True
                    | TokenKind::Colon
                    | TokenKind::Indent
                    | TokenKind::Pass
                    | TokenKind::Dedent
            )
        })
        .collect();

    let expected = vec![
        TokenKind::If,
        TokenKind::True,
        TokenKind::Colon,
        TokenKind::Indent,
        TokenKind::Pass,
        TokenKind::Dedent,
    ];

    assert_eq!(relevant, expected);
}

#[test]
fn test_comments() {
    let code = "# comment\nx = 1 # inline";
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.scan_tokens();

    let kinds: Vec<TokenKind> = tokens
        .iter()
        .map(|t| t.kind.clone())
        .filter(|k| !matches!(k, TokenKind::Newline | TokenKind::Eof))
        .collect();

    // Comments are skipped.
    // Line 1: # comment -> skipped. Newline (filtered).
    // Line 2: x = 1. # inline -> skipped.

    let expected = vec![
        TokenKind::Identifier("x".to_string()),
        TokenKind::Assign,
        TokenKind::Integer(1),
    ];

    assert_eq!(kinds, expected);
}
