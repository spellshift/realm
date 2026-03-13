use eldritch_core::{Span, TokenKind};

#[test]
fn test_span_creation() {
    let span = Span::new(1, 2, 3);
    assert_eq!(span.start, 1);
    assert_eq!(span.end, 2);
    assert_eq!(span.line, 3);
}

#[test]
fn test_tokenkind_display() {
    let cases = vec![
        (TokenKind::LParen, "("),
        (TokenKind::RParen, ")"),
        (TokenKind::LBracket, "["),
        (TokenKind::RBracket, "]"),
        (TokenKind::LBrace, "{"),
        (TokenKind::RBrace, "}"),
        (TokenKind::Comma, ","),
        (TokenKind::Colon, ":"),
        (TokenKind::Dot, "."),
        (TokenKind::Minus, "-"),
        (TokenKind::Plus, "+"),
        (TokenKind::Star, "*"),
        (TokenKind::Slash, "/"),
        (TokenKind::Percent, "%"),
        (TokenKind::Assign, "="),
        (TokenKind::Newline, "newline"),
        (TokenKind::BitAnd, "&"),
        (TokenKind::BitOr, "|"),
        (TokenKind::BitXor, "^"),
        (TokenKind::BitNot, "~"),
        (TokenKind::LShift, "<<"),
        (TokenKind::RShift, ">>"),
        (TokenKind::StarStar, "**"),
        (TokenKind::SlashSlash, "//"),
        (TokenKind::PlusAssign, "+="),
        (TokenKind::MinusAssign, "-="),
        (TokenKind::StarAssign, "*="),
        (TokenKind::SlashAssign, "/="),
        (TokenKind::PercentAssign, "%="),
        (TokenKind::SlashSlashAssign, "//="),
        (TokenKind::Arrow, "->"),
        (TokenKind::Eq, "=="),
        (TokenKind::NotEq, "!="),
        (TokenKind::Lt, "<"),
        (TokenKind::Gt, ">"),
        (TokenKind::LtEq, "<="),
        (TokenKind::GtEq, ">="),
        (TokenKind::Def, "def"),
        (TokenKind::If, "if"),
        (TokenKind::Elif, "elif"),
        (TokenKind::Else, "else"),
        (TokenKind::Return, "return"),
        (TokenKind::For, "for"),
        (TokenKind::In, "in"),
        (TokenKind::NotIn, "not in"),
        (TokenKind::True, "True"),
        (TokenKind::False, "False"),
        (TokenKind::None, "None"),
        (TokenKind::And, "and"),
        (TokenKind::Or, "or"),
        (TokenKind::Not, "not"),
        (TokenKind::Break, "break"),
        (TokenKind::Continue, "continue"),
        (TokenKind::Pass, "pass"),
        (TokenKind::Lambda, "lambda"),
        (TokenKind::Indent, "indent"),
        (TokenKind::Dedent, "dedent"),
        (TokenKind::Eof, "EOF"),
    ];

    for (token, expected) in cases {
        assert_eq!(format!("{}", token), expected);
    }

    // Literals and Error
    assert_eq!(
        format!("{}", TokenKind::Identifier(String::from("test"))),
        "identifier \"test\""
    );
    assert_eq!(
        format!("{}", TokenKind::String(String::from("test"))),
        "string \"test\""
    );
    assert_eq!(format!("{}", TokenKind::Bytes(Vec::new())), "bytes");
    assert_eq!(format!("{}", TokenKind::Integer(42)), "42");
    assert_eq!(format!("{}", TokenKind::Float(3.14)), "3.14");
    assert_eq!(
        format!("{}", TokenKind::FStringContent(Vec::new())),
        "f-string"
    );
    assert_eq!(
        format!("{}", TokenKind::Error(String::from("err"))),
        "Error: err"
    );
}

#[test]
fn test_tokenkind_from_keyword() {
    let keywords = vec![
        ("def", TokenKind::Def),
        ("if", TokenKind::If),
        ("elif", TokenKind::Elif),
        ("else", TokenKind::Else),
        ("return", TokenKind::Return),
        ("for", TokenKind::For),
        ("in", TokenKind::In),
        ("True", TokenKind::True),
        ("False", TokenKind::False),
        ("None", TokenKind::None),
        ("and", TokenKind::And),
        ("or", TokenKind::Or),
        ("not", TokenKind::Not),
        ("break", TokenKind::Break),
        ("continue", TokenKind::Continue),
        ("pass", TokenKind::Pass),
        ("lambda", TokenKind::Lambda),
    ];

    for (s, expected) in keywords {
        assert_eq!(TokenKind::from_keyword(s), Some(expected));
    }

    assert_eq!(TokenKind::from_keyword("invalid"), None);
    assert_eq!(TokenKind::from_keyword(""), None);
}
