use eldritch_core::{Lexer, Span, Token, TokenKind};

// Helper to strip spans from tokens for easier comparison
fn strip_spans(tokens: Vec<Token>) -> Vec<TokenKind> {
    tokens
        .into_iter()
        .map(|t| match t.kind {
            TokenKind::FStringContent(inner) => {
                let stripped_inner = inner
                    .into_iter()
                    .map(|it| Token {
                        kind: match it.kind {
                            TokenKind::FStringContent(_) => {
                                panic!("Nested FStringContent not supported in stripper")
                            }
                            k => k,
                        },
                        span: Span::new(0, 0, 0),
                    })
                    .collect();
                TokenKind::FStringContent(stripped_inner)
            }
            k => k,
        })
        .collect()
}

fn assert_lex(source: &str, expected: Vec<TokenKind>) {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();
    let kinds = strip_spans(tokens);
    assert_eq!(kinds, expected, "Source: {:?}", source);
}

#[test]
fn test_identifiers_keywords() {
    // Keywords
    assert_lex(
        "def if else return",
        vec![
            TokenKind::Def,
            TokenKind::If,
            TokenKind::Else,
            TokenKind::Return,
            TokenKind::Newline, // implicitly added
            TokenKind::Eof,
        ],
    );

    // Identifiers
    assert_lex(
        "variable name_with_underscore _private",
        vec![
            TokenKind::Identifier("variable".to_string()),
            TokenKind::Identifier("name_with_underscore".to_string()),
            TokenKind::Identifier("_private".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );

    // Mixed
    assert_lex(
        "for x in items",
        vec![
            TokenKind::For,
            TokenKind::Identifier("x".to_string()),
            TokenKind::In,
            TokenKind::Identifier("items".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_numbers() {
    assert_lex(
        "123 45.67 .89 0",
        vec![
            TokenKind::Integer(123),
            TokenKind::Float(45.67),
            TokenKind::Float(0.89),
            TokenKind::Integer(0),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_strings_basic() {
    assert_lex(
        r#"'single' "double""#,
        vec![
            TokenKind::String("single".to_string()),
            TokenKind::String("double".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_strings_escapes() {
    // Note: The lexer processes escapes.
    // "hello\nworld" -> String("hello\nworld")
    assert_lex(
        r#""hello\nworld""#,
        vec![
            TokenKind::String("hello\nworld".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );

    // Quote escaping
    assert_lex(
        r#"'It\'s me'"#,
        vec![
            TokenKind::String("It's me".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_raw_strings() {
    // Raw strings preserve backslashes
    assert_lex(
        r#"r"path\to\file""#,
        vec![
            TokenKind::String(r"path\to\file".to_string()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_bytes_strings() {
    assert_lex(
        r#"b"hello""#,
        vec![
            TokenKind::Bytes(b"hello".to_vec()),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_f_strings_simple() {
    // f"value: {x}"
    // Should produce FStringContent containing [String("value: "), LParen, Identifier(x), RParen]
    // Note: Lexer wraps expressions in parens for f-strings

    // We need to manually construct the inner tokens with dummy spans because our stripper sets them to (0,0,0)
    let inner_x = vec![
        Token {
            kind: TokenKind::String("val: ".to_string()),
            span: Span::new(0, 0, 0),
        },
        Token {
            kind: TokenKind::LParen,
            span: Span::new(0, 0, 0),
        },
        Token {
            kind: TokenKind::Identifier("x".to_string()),
            span: Span::new(0, 0, 0),
        },
        Token {
            kind: TokenKind::RParen,
            span: Span::new(0, 0, 0),
        },
    ];

    assert_lex(
        r#"f"val: {x}""#,
        vec![
            TokenKind::FStringContent(inner_x),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_operators() {
    assert_lex(
        "+ - * / % ** //",
        vec![
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::StarStar,
            TokenKind::SlashSlash,
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );

    assert_lex(
        "+= -= *= /= %= **= //=",
        vec![
            TokenKind::PlusAssign,
            TokenKind::MinusAssign,
            TokenKind::StarAssign,
            TokenKind::SlashAssign,
            TokenKind::PercentAssign,
            // Note: StarStarAssign (pow assign) is not currently implemented as a single token
            TokenKind::StarStar,
            TokenKind::Assign,
            TokenKind::SlashSlashAssign,
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );

    assert_lex(
        "== != < > <= >= ->",
        vec![
            TokenKind::Eq,
            TokenKind::NotEq,
            TokenKind::Lt,
            TokenKind::Gt,
            TokenKind::LtEq,
            TokenKind::GtEq,
            TokenKind::Arrow,
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );

    assert_lex(
        "& | ^ ~ << >>",
        vec![
            TokenKind::BitAnd,
            TokenKind::BitOr,
            TokenKind::BitXor,
            TokenKind::BitNot,
            TokenKind::LShift,
            TokenKind::RShift,
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_punctuation() {
    assert_lex(
        "() [] {} , : .",
        vec![
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBracket,
            TokenKind::RBracket,
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::Comma,
            TokenKind::Colon,
            TokenKind::Dot,
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_comments() {
    assert_lex(
        "x = 1 # assignment",
        vec![
            TokenKind::Identifier("x".to_string()),
            TokenKind::Assign,
            TokenKind::Integer(1),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );

    assert_lex(
        "# just a comment\n",
        vec![TokenKind::Newline, TokenKind::Eof],
    );
}

#[test]
fn test_indentation() {
    // if True:
    //   pass
    // x = 1
    let source = "if True:\n  pass\nx = 1";
    assert_lex(
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
            TokenKind::Identifier("x".to_string()),
            TokenKind::Assign,
            TokenKind::Integer(1),
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_nested_indentation() {
    // if True:
    //   if False:
    //     pass
    let source = "if True:\n  if False:\n    pass";
    assert_lex(
        source,
        vec![
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
        ],
    );
}
