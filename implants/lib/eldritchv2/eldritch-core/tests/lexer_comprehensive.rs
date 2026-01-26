use eldritch_core::{Lexer, TokenKind};

fn scan(input: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(input.to_string());
    lexer.scan_tokens().into_iter().map(|t| t.kind).collect()
}

#[test]
fn test_identifiers_and_keywords() {
    let input = "abc def if else True False None";
    let tokens = scan(input);
    let expected = vec![
        TokenKind::Identifier("abc".to_string()),
        TokenKind::Def,
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
    let input = "123 45.67 .5 0";
    let tokens = scan(input);
    let expected = vec![
        TokenKind::Integer(123),
        TokenKind::Float(45.67),
        TokenKind::Float(0.5),
        TokenKind::Integer(0),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_strings() {
    let input = r#""hello" 'world' b"bytes" r"raw""#;
    let tokens = scan(input);
    let expected = vec![
        TokenKind::String("hello".to_string()),
        TokenKind::String("world".to_string()),
        TokenKind::Bytes(b"bytes".to_vec()),
        TokenKind::String("raw".to_string()), // raw strings are just strings in TokenKind
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_operators() {
    let input = "+ - * / % ** // ->";
    let tokens = scan(input);
    let expected = vec![
        TokenKind::Plus,
        TokenKind::Minus,
        TokenKind::Star,
        TokenKind::Slash,
        TokenKind::Percent,
        TokenKind::StarStar,
        TokenKind::SlashSlash,
        TokenKind::Arrow,
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_augmented_assignment() {
    let input = "+= -= *= /= %= //=";
    let tokens = scan(input);
    let expected = vec![
        TokenKind::PlusAssign,
        TokenKind::MinusAssign,
        TokenKind::StarAssign,
        TokenKind::SlashAssign,
        TokenKind::PercentAssign,
        TokenKind::SlashSlashAssign,
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_comparison() {
    let input = "== != < > <= >=";
    let tokens = scan(input);
    let expected = vec![
        TokenKind::Eq,
        TokenKind::NotEq,
        TokenKind::Lt,
        TokenKind::Gt,
        TokenKind::LtEq,
        TokenKind::GtEq,
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_bitwise() {
    let input = "& | ^ ~ << >>";
    let tokens = scan(input);
    let expected = vec![
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
fn test_delimiters() {
    let input = "( ) [ ] { } , : .";
    let tokens = scan(input);
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
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_comments_and_newlines() {
    let input = "a # comment\nb";
    let tokens = scan(input);
    // Comment consumes until newline, then emits newline
    let expected = vec![
        TokenKind::Identifier("a".to_string()),
        TokenKind::Newline,
        TokenKind::Identifier("b".to_string()),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_indentation() {
    let input = "if True:
    x
    y
z
";
    let tokens = scan(input);
    let expected = vec![
        TokenKind::If,
        TokenKind::True,
        TokenKind::Colon,
        TokenKind::Newline,
        TokenKind::Indent,
        TokenKind::Identifier("x".to_string()),
        TokenKind::Newline,
        TokenKind::Identifier("y".to_string()),
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Identifier("z".to_string()),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_fstrings() {
    let input = "f\"val={x}\"";
    let tokens = scan(input);
    // This depends on how f-strings are tokenized.
    // Based on lexer.rs:
    // f"val={x}" -> FStringContent([String("val="), LParen, Identifier("x"), RParen])
    // The inner tokens have spans, which makes exact matching hard if we only strip outer spans.
    // But `scan` strips kinds of the top level tokens.
    // Inside `FStringContent`, it holds `Vec<Token>`, which has spans.

    // We need to inspect the result and relax span checks for inner tokens
    // or manually construct what we expect.

    // Let's look at the result manually.
    let result = tokens;
    assert_eq!(result.len(), 3); // FStringContent + Newline + Eof

    match &result[0] {
        TokenKind::FStringContent(inner_tokens) => {
            // Check inner structure
            assert_eq!(inner_tokens.len(), 4);
            match &inner_tokens[0].kind {
                TokenKind::String(s) => assert_eq!(s, "val="),
                _ => panic!("Expected string val="),
            }
            assert_eq!(inner_tokens[1].kind, TokenKind::LParen);
            match &inner_tokens[2].kind {
                TokenKind::Identifier(s) => assert_eq!(s, "x"),
                _ => panic!("Expected identifier x"),
            }
            assert_eq!(inner_tokens[3].kind, TokenKind::RParen);
        }
        _ => panic!("Expected FStringContent"),
    }
}

#[test]
fn test_fstring_expression() {
    let input = "f\"{1+2}\"";
    let tokens = scan(input);

    match &tokens[0] {
        TokenKind::FStringContent(inner_tokens) => {
            // {1+2} -> LParen, Integer(1), Plus, Integer(2), RParen
            assert_eq!(inner_tokens.len(), 5);
            assert_eq!(inner_tokens[0].kind, TokenKind::LParen);
            assert_eq!(inner_tokens[1].kind, TokenKind::Integer(1));
            assert_eq!(inner_tokens[2].kind, TokenKind::Plus);
            assert_eq!(inner_tokens[3].kind, TokenKind::Integer(2));
            assert_eq!(inner_tokens[4].kind, TokenKind::RParen);
        }
        _ => panic!("Expected FStringContent"),
    }
}

#[test]
fn test_triple_quoted_string() {
    let input = r#""""
line1
line2
""""#;
    let tokens = scan(input);
    let expected = vec![
        TokenKind::String("\nline1\nline2\n".to_string()),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}
