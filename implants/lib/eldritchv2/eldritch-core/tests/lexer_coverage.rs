use eldritch_core::{Lexer, TokenKind};

fn assert_tokens(source: &str, expected: Vec<TokenKind>) {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    let token_kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();

    if token_kinds != expected {
        panic!(
            "Token mismatch.\nSource: {:?}\nExpected: {:?}\nActual:   {:?}",
            source, expected, token_kinds
        );
    }
}

#[test]
fn test_numbers() {
    assert_tokens(
        "123",
        vec![TokenKind::Integer(123), TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "123.456",
        vec![
            TokenKind::Float(123.456),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
    assert_tokens(
        ".5",
        vec![TokenKind::Float(0.5), TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "0",
        vec![TokenKind::Integer(0), TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "0.0",
        vec![TokenKind::Float(0.0), TokenKind::Newline, TokenKind::Eof],
    );

    // Negative numbers are tokenized as Minus + Number in the lexer
    assert_tokens(
        "-123",
        vec![
            TokenKind::Minus,
            TokenKind::Integer(123),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_strings() {
    // Normal strings
    assert_tokens(
        r#""hello""#,
        vec![
            TokenKind::String("hello".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
    assert_tokens(
        r#"'hello'"#,
        vec![
            TokenKind::String("hello".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );

    // Escapes
    assert_tokens(
        r#""a\nb""#,
        vec![
            TokenKind::String("a\nb".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );

    // Raw strings
    assert_tokens(
        r#"r"a\nb""#,
        vec![
            TokenKind::String("a\\nb".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );

    // Bytes
    assert_tokens(
        r#"b"abc""#,
        vec![
            TokenKind::Bytes(vec![97, 98, 99]),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );

    // Triple quoted
    assert_tokens(
        r#""""multiline
string""""#,
        vec![
            TokenKind::String("multiline\nstring".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );

    // Empty strings
    assert_tokens(
        r#""""#,
        vec![
            TokenKind::String("".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
    assert_tokens(
        r#"''"#,
        vec![
            TokenKind::String("".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_identifiers_and_keywords() {
    assert_tokens(
        "abc",
        vec![
            TokenKind::Identifier("abc".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
    assert_tokens(
        "_abc",
        vec![
            TokenKind::Identifier("_abc".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
    assert_tokens(
        "abc1",
        vec![
            TokenKind::Identifier("abc1".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );

    // Keywords
    assert_tokens(
        "def",
        vec![TokenKind::Def, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "if",
        vec![TokenKind::If, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "return",
        vec![TokenKind::Return, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "True",
        vec![TokenKind::True, TokenKind::Newline, TokenKind::Eof],
    );
}

#[test]
fn test_operators() {
    assert_tokens(
        "+",
        vec![TokenKind::Plus, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "+=",
        vec![TokenKind::PlusAssign, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "-",
        vec![TokenKind::Minus, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "-=",
        vec![TokenKind::MinusAssign, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "->",
        vec![TokenKind::Arrow, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "*",
        vec![TokenKind::Star, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "**",
        vec![TokenKind::StarStar, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "*=",
        vec![TokenKind::StarAssign, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "/",
        vec![TokenKind::Slash, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "//",
        vec![TokenKind::SlashSlash, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "/=",
        vec![TokenKind::SlashAssign, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "//=",
        vec![
            TokenKind::SlashSlashAssign,
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );

    assert_tokens(
        "==",
        vec![TokenKind::Eq, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        "!=",
        vec![TokenKind::NotEq, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens("<", vec![TokenKind::Lt, TokenKind::Newline, TokenKind::Eof]);
    assert_tokens(
        "<=",
        vec![TokenKind::LtEq, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(">", vec![TokenKind::Gt, TokenKind::Newline, TokenKind::Eof]);
    assert_tokens(
        ">=",
        vec![TokenKind::GtEq, TokenKind::Newline, TokenKind::Eof],
    );

    assert_tokens(
        "<<",
        vec![TokenKind::LShift, TokenKind::Newline, TokenKind::Eof],
    );
    assert_tokens(
        ">>",
        vec![TokenKind::RShift, TokenKind::Newline, TokenKind::Eof],
    );
}

#[test]
fn test_indentation() {
    let source = "if True:
    pass
";
    assert_tokens(
        source,
        vec![
            TokenKind::If,
            TokenKind::True,
            TokenKind::Colon,
            TokenKind::Newline,
            TokenKind::Indent,
            TokenKind::Pass,
            TokenKind::Newline,
            TokenKind::Dedent,
            TokenKind::Eof,
        ],
    );

    let source_dedent = "if True:
    pass
pass
";
    assert_tokens(
        source_dedent,
        vec![
            TokenKind::If,
            TokenKind::True,
            TokenKind::Colon,
            TokenKind::Newline,
            TokenKind::Indent,
            TokenKind::Pass,
            TokenKind::Newline,
            TokenKind::Dedent,
            TokenKind::Pass,
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_comments() {
    // Lexer adds a newline at the end, so comment ending at EOF (or implicit newline) produces a Newline token
    assert_tokens("# comment", vec![TokenKind::Newline, TokenKind::Eof]);
    assert_tokens(
        "1 # comment",
        vec![TokenKind::Integer(1), TokenKind::Newline, TokenKind::Eof],
    );

    let source = "
# line 1
1
# line 3
";
    // This source starts with a newline!
    assert_tokens(
        source,
        vec![
            TokenKind::Newline, // from the leading \n
            // # line 1\n -> consumed, emits Newline
            TokenKind::Newline,
            TokenKind::Integer(1),
            TokenKind::Newline, // after 1
            // # line 3\n -> consumed, emits Newline
            TokenKind::Newline,
            TokenKind::Eof],
    );
}

#[test]
fn test_errors() {
    // Unterminated string
    let source = "\"hello";
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    assert!(matches!(tokens[0].kind, TokenKind::Error(_)));

    // Unexpected char
    let source = "?";
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    assert!(matches!(tokens[0].kind, TokenKind::Error(_)));
}

#[test]
fn test_f_strings() {
    // Basic f-string
    let source = "f\"val={x}\"";
    // Based on lexer logic, this should produce:
    // FStringContent([
    //   String("val="),
    //   LParen, Identifier("x"), RParen
    // ])
    // But verify the recursive tokenization structure

    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();

    // We expect one FStringContent token followed by Newline, Eof
    assert_eq!(tokens.len(), 3);
    match &tokens[0].kind {
        TokenKind::FStringContent(inner_tokens) => {
            // "val=" part
            assert!(matches!(inner_tokens[0].kind, TokenKind::String(_)));
            // {x} part -> LParen, Identifier(x), RParen
            assert_eq!(inner_tokens[1].kind, TokenKind::LParen);
            match &inner_tokens[2].kind {
                TokenKind::Identifier(s) => assert_eq!(s, "x"),
                _ => panic!("Expected identifier x"),
            }
            assert_eq!(inner_tokens[3].kind, TokenKind::RParen);
        }
        _ => panic!("Expected FStringContent, got {:?}", tokens[0].kind),
    }
}
