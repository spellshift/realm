extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec;
use eldritch_core::{Span, TokenKind};

#[test]
fn test_token_kind_display() {
    let tokens = [
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

    for (kind, expected) in tokens {
        assert_eq!(format!("{}", kind), expected);
    }
}

#[test]
fn test_token_kind_display_with_data() {
    assert_eq!(
        format!("{}", TokenKind::Identifier(String::from("foo"))),
        "identifier \"foo\""
    );
    assert_eq!(
        format!("{}", TokenKind::String(String::from("bar"))),
        "string \"bar\""
    );
    assert_eq!(format!("{}", TokenKind::Bytes(vec![0x00, 0x01])), "bytes");
    assert_eq!(format!("{}", TokenKind::Integer(42)), "42");
    assert_eq!(format!("{}", TokenKind::Float(3.14)), "3.14");
    assert_eq!(format!("{}", TokenKind::FStringContent(vec![])), "f-string");
    assert_eq!(
        format!("{}", TokenKind::Error(String::from("oops"))),
        "Error: oops"
    );
}

#[test]
fn test_span_new() {
    let span = Span::new(10, 20, 5);
    assert_eq!(span.start, 10);
    assert_eq!(span.end, 20);
    assert_eq!(span.line, 5);
}

#[test]
fn test_token_kind_from_keyword() {
    let keywords = [
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

    for (kw, kind) in keywords {
        assert_eq!(TokenKind::from_keyword(kw), Some(kind));
    }

    assert_eq!(TokenKind::from_keyword("unknown_keyword"), None);
}
