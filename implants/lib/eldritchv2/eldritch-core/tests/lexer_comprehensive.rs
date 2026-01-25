use eldritch_core::{Lexer, TokenKind};

fn scan(input: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(input.to_string());
    lexer.scan_tokens().into_iter().map(|t| t.kind).collect()
}

#[test]
fn test_identifiers_and_keywords() {
    let tokens = scan("def foo if True");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Def,
            TokenKind::Identifier("foo".to_string()),
            TokenKind::If,
            TokenKind::True,
            TokenKind::Newline,
            TokenKind::Eof
        ]
    );

    let tokens = scan("_var var123 VAR");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Identifier("_var".to_string()),
            TokenKind::Identifier("var123".to_string()),
            TokenKind::Identifier("VAR".to_string()),
            TokenKind::Newline,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_numbers() {
    let tokens = scan("123 12.34 .5 0");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Integer(123),
            TokenKind::Float(12.34),
            TokenKind::Float(0.5),
            TokenKind::Integer(0),
            TokenKind::Newline,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_strings() {
    let tokens = scan(r#"'single' "double""#);
    assert_eq!(
        tokens,
        vec![
            TokenKind::String("single".to_string()),
            TokenKind::String("double".to_string()),
            TokenKind::Newline,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_raw_strings() {
    let tokens = scan(r#"r"raw\n" r'raw\t'"#);
    assert_eq!(
        tokens,
        vec![
            TokenKind::String("raw\\n".to_string()),
            TokenKind::String("raw\\t".to_string()),
            TokenKind::Newline,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_bytes() {
    let tokens = scan(r#"b"hello" b'world'"#);
    assert_eq!(
        tokens,
        vec![
            TokenKind::Bytes(b"hello".to_vec()),
            TokenKind::Bytes(b"world".to_vec()),
            TokenKind::Newline,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_f_strings() {
    // Simple f-string without expressions
    let tokens = scan(r#"f"hello""#);
    if let TokenKind::FStringContent(inner) = &tokens[0] {
        assert_eq!(inner.len(), 1);
        assert_eq!(inner[0].kind, TokenKind::String("hello".to_string()));
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0]);
    }
}

#[test]
fn test_f_strings_flexible() {
    let tokens = scan(r#"f"hello {name}""#);
    // f"hello {name}" -> FStringContent([String("hello "), LParen, Identifier("name"), RParen])
    // The exact token structure inside FStringContent is complex.

    if let TokenKind::FStringContent(inner) = &tokens[0] {
        // inner should be: String("hello "), LParen, Identifier("name"), RParen
        // The lexer produces:
        // String("hello ")
        // LParen
        // Identifier("name")
        // RParen
        // Note: The lexer recursively tokenizes the expression.

        let kinds: Vec<TokenKind> = inner.iter().map(|t| t.kind.clone()).collect();
        assert_eq!(kinds[0], TokenKind::String("hello ".to_string()));
        assert_eq!(kinds[1], TokenKind::LParen);
        assert_eq!(kinds[2], TokenKind::Identifier("name".to_string()));
        assert_eq!(kinds[3], TokenKind::RParen);
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0]);
    }
}

#[test]
fn test_operators_and_delimiters() {
    let tokens = scan("+ - * / % ** // << >> & | ^ ~ < > <= >= == != = += -= *= /= %= //= ->");
    let expected = vec![
        TokenKind::Plus,
        TokenKind::Minus,
        TokenKind::Star,
        TokenKind::Slash,
        TokenKind::Percent,
        TokenKind::StarStar,
        TokenKind::SlashSlash,
        TokenKind::LShift,
        TokenKind::RShift,
        TokenKind::BitAnd,
        TokenKind::BitOr,
        TokenKind::BitXor,
        TokenKind::BitNot,
        TokenKind::Lt,
        TokenKind::Gt,
        TokenKind::LtEq,
        TokenKind::GtEq,
        TokenKind::Eq,
        TokenKind::NotEq,
        TokenKind::Assign,
        TokenKind::PlusAssign,
        TokenKind::MinusAssign,
        TokenKind::StarAssign,
        TokenKind::SlashAssign,
        TokenKind::PercentAssign,
        TokenKind::SlashSlashAssign,
        TokenKind::Arrow,
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);

    let tokens = scan("() [] {} , : . ;"); // ; produces Newline
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
        TokenKind::Newline, // from ;
        TokenKind::Newline, // from implicit end
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_comments() {
    let tokens = scan("# this is a comment\nx = 1 # inline comment");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Newline, // The comment consumes until newline, then emits Newline
            TokenKind::Identifier("x".to_string()),
            TokenKind::Assign,
            TokenKind::Integer(1),
            TokenKind::Newline, // Inline comment ends with newline
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_indentation() {
    let tokens = scan("def f():\n  pass");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Def,
            TokenKind::Identifier("f".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::Colon,
            TokenKind::Newline,
            TokenKind::Indent,
            TokenKind::Pass,
            TokenKind::Newline,
            TokenKind::Dedent,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_triple_quoted_strings() {
    let code = r#""""multi
line""""#;
    let tokens = scan(code);
    assert_eq!(
        tokens,
        vec![
            TokenKind::String("multi\nline".to_string()),
            TokenKind::Newline,
            TokenKind::Eof
        ]
    );
}

#[test]
fn test_error_tokens() {
    // Unterminated string
    let tokens = scan("\"terminated");
    if let TokenKind::Error(msg) = &tokens[0] {
        assert!(msg.contains("Unterminated string"));
    } else {
        panic!("Expected Error token, got {:?}", tokens[0]);
    }

    // Unexpected character
    let tokens = scan("?");
    if let TokenKind::Error(msg) = &tokens[0] {
        assert!(msg.contains("Unexpected character"));
    } else {
        panic!("Expected Error token, got {:?}", tokens[0]);
    }
}
