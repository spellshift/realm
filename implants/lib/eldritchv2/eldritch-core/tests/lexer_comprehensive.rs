use eldritch_core::{Lexer, TokenKind};

fn assert_tokens(source: &str, expected: Vec<TokenKind>) {
    let mut lexer = Lexer::new(String::from(source));
    let tokens = lexer.scan_tokens();
    let token_kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();

    assert_eq!(
        token_kinds, expected,
        "Tokens mismatch for source: {:?}",
        source
    );
}

#[test]
fn test_identifiers_and_keywords() {
    let inputs: Vec<(&str, Vec<TokenKind>)> = vec![
        ("abc", vec![TokenKind::Identifier("abc".into())]),
        ("def", vec![TokenKind::Def]),
        ("if", vec![TokenKind::If]),
        ("elif", vec![TokenKind::Elif]),
        ("else", vec![TokenKind::Else]),
        ("return", vec![TokenKind::Return]),
        (
            "while_loop",
            vec![TokenKind::Identifier("while_loop".into())],
        ),
    ];

    for (src, mut expected_kinds) in inputs {
        // Lexer implicitly adds a Newline (because it appends \n to source) and Eof
        expected_kinds.push(TokenKind::Newline);
        expected_kinds.push(TokenKind::Eof);
        assert_tokens(src, expected_kinds);
    }
}

#[test]
fn test_numbers() {
    let inputs: Vec<(&str, Vec<TokenKind>)> = vec![
        ("123", vec![TokenKind::Integer(123)]),
        ("123.456", vec![TokenKind::Float(123.456)]),
        (".5", vec![TokenKind::Float(0.5)]),
        ("0", vec![TokenKind::Integer(0)]),
    ];

    for (src, mut expected_kinds) in inputs {
        expected_kinds.push(TokenKind::Newline);
        expected_kinds.push(TokenKind::Eof);
        assert_tokens(src, expected_kinds);
    }

    // "1." is ambiguous in some lexers, let's see how this one handles it.
    // . number() method:
    // if self.peek() == '.' && self.peek_next().is_ascii_digit() { ... }
    // So "1." -> "1" is integer, "." is dot.
    assert_tokens(
        "1.",
        vec![
            TokenKind::Integer(1),
            TokenKind::Dot,
            TokenKind::Newline,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn test_strings() {
    let inputs: Vec<(&str, Vec<TokenKind>)> = vec![
        ("'hello'", vec![TokenKind::String("hello".into())]),
        ("\"world\"", vec![TokenKind::String("world".into())]),
        ("'a\\nb'", vec![TokenKind::String("a\nb".into())]),
    ];

    for (src, mut expected_kinds) in inputs {
        expected_kinds.push(TokenKind::Newline);
        expected_kinds.push(TokenKind::Eof);
        assert_tokens(src, expected_kinds);
    }
}

#[test]
fn test_raw_strings() {
    let inputs: Vec<(&str, Vec<TokenKind>)> = vec![
        (
            "r'hello\\nworld'",
            vec![TokenKind::String("hello\\nworld".into())],
        ),
        (
            "r\"path\\to\\file\"",
            vec![TokenKind::String("path\\to\\file".into())],
        ),
    ];

    for (src, mut expected_kinds) in inputs {
        expected_kinds.push(TokenKind::Newline);
        expected_kinds.push(TokenKind::Eof);
        assert_tokens(src, expected_kinds);
    }
}

#[test]
fn test_bytes() {
    let inputs: Vec<(&str, Vec<TokenKind>)> =
        vec![("b'bytes'", vec![TokenKind::Bytes(b"bytes".to_vec())])];

    for (src, mut expected_kinds) in inputs {
        expected_kinds.push(TokenKind::Newline);
        expected_kinds.push(TokenKind::Eof);
        assert_tokens(src, expected_kinds);
    }
}

#[test]
fn test_operators() {
    let src = "+ - * / % ** // = == != < > <= >= -> += -= *= /= %= //= & | ^ ~ << >>";
    let expected = vec![
        TokenKind::Plus,
        TokenKind::Minus,
        TokenKind::Star,
        TokenKind::Slash,
        TokenKind::Percent,
        TokenKind::StarStar,
        TokenKind::SlashSlash,
        TokenKind::Assign,
        TokenKind::Eq,
        TokenKind::NotEq,
        TokenKind::Lt,
        TokenKind::Gt,
        TokenKind::LtEq,
        TokenKind::GtEq,
        TokenKind::Arrow,
        TokenKind::PlusAssign,
        TokenKind::MinusAssign,
        TokenKind::StarAssign,
        TokenKind::SlashAssign,
        TokenKind::PercentAssign,
        TokenKind::SlashSlashAssign,
        TokenKind::BitAnd,
        TokenKind::BitOr,
        TokenKind::BitXor,
        TokenKind::BitNot,
        TokenKind::LShift,
        TokenKind::RShift,
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_tokens(src, expected);
}

#[test]
fn test_fstrings() {
    let src = "f'val={x}'";
    let mut lexer = Lexer::new(String::from(src));
    let tokens = lexer.scan_tokens();

    assert_eq!(tokens.len(), 3); // FStringContent, Newline, Eof

    if let TokenKind::FStringContent(inner_tokens) = &tokens[0].kind {
        // inner_tokens: [String("val="), LParen, Identifier("x"), RParen]
        let kinds: Vec<TokenKind> = inner_tokens.iter().map(|t| t.kind.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::String("val=".into()),
                TokenKind::LParen,
                TokenKind::Identifier("x".into()),
                TokenKind::RParen,
            ]
        );
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0].kind);
    }
}

#[test]
fn test_indentation() {
    let src_block = "if True:\n    x";
    let expected = vec![
        TokenKind::If,
        TokenKind::True,
        TokenKind::Colon,
        TokenKind::Newline,
        TokenKind::Indent,
        TokenKind::Identifier("x".into()),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ];

    assert_tokens(src_block, expected);
}

#[test]
fn test_comments() {
    let src = "x # comment\ny";
    let expected = vec![
        TokenKind::Identifier("x".into()),
        TokenKind::Newline,
        TokenKind::Identifier("y".into()),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_tokens(src, expected);
}
