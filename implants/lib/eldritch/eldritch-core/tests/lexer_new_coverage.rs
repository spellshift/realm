use eldritch_core::{Lexer, TokenKind};

fn lex(source: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(String::from(source));
    lexer.scan_tokens().into_iter().map(|t| t.kind).collect()
}

#[test]
fn test_fstring_nested_braces() {
    // Tests that { {x} } is tokenized as an expression containing braces
    // This confirms that {{ is NOT an escape sequence for {, but rather a nested structure or just braces in the expression.
    let input = "f\"{ {x} }\"";
    let tokens = lex(input);

    if let TokenKind::FStringContent(inner_tokens) = &tokens[0] {
        let kinds: Vec<TokenKind> = inner_tokens.iter().map(|t| t.kind.clone()).collect();
        // The expression source is " {x} "
        // " " -> ignored
        // "{" -> LBrace
        // "x" -> Identifier
        // "}" -> RBrace
        // " " -> ignored
        let expected = vec![
            TokenKind::LParen,
            TokenKind::LBrace,
            TokenKind::Identifier(String::from("x")),
            TokenKind::RBrace,
            TokenKind::RParen,
        ];
        assert_eq!(kinds, expected, "Nested braces in f-string failed");
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0]);
    }
}

#[test]
fn test_fstring_multiple_expressions() {
    let input = "f\"{x} + {y}\"";
    let tokens = lex(input);

    if let TokenKind::FStringContent(inner_tokens) = &tokens[0] {
        let kinds: Vec<TokenKind> = inner_tokens.iter().map(|t| t.kind.clone()).collect();
        let expected = vec![
            // {x}
            TokenKind::LParen,
            TokenKind::Identifier(String::from("x")),
            TokenKind::RParen,
            // " + "
            TokenKind::String(String::from(" + ")),
            // {y}
            TokenKind::LParen,
            TokenKind::Identifier(String::from("y")),
            TokenKind::RParen,
        ];
        assert_eq!(kinds, expected, "Multiple expressions in f-string failed");
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0]);
    }
}

#[test]
fn test_fstring_empty_expression() {
    // f"{}" -> expression is empty string -> lexer returns empty -> wrapped in parens?
    // Let's see tokenize_fstring_expression:
    // Lexer::new("").scan_tokens() -> [Eof]
    // loop matches Eof -> break
    // returns [LParen, RParen]
    let input = "f\"{}\"";
    let tokens = lex(input);

    if let TokenKind::FStringContent(inner_tokens) = &tokens[0] {
        let kinds: Vec<TokenKind> = inner_tokens.iter().map(|t| t.kind.clone()).collect();
        let expected = vec![TokenKind::LParen, TokenKind::RParen];
        assert_eq!(kinds, expected, "Empty expression in f-string failed");
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0]);
    }
}

#[test]
fn test_compound_operators_split() {
    // Current lexer implementation does not combine these into single tokens
    let input = "**= &= |= ^= <<= >>=";
    let tokens = lex(input);
    let expected = vec![
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
    assert_eq!(tokens, expected, "Compound operators split check failed");
}

#[test]
fn test_raw_string_backslash_end() {
    // r"\" -> raw string containing backslash?
    // logic: is_raw=true. peek quote? no.
    // c == '\\'. advance.
    // is_raw=true.
    // next_char == quote? if input is r"\"", next is ".
    // matches next_char == quote_char -> escape quote, keep backslash.
    // current_literal.push('\\'); current_literal.push('"');

    // Test case: r"\"" should be string `\"`
    let input = "r\"\\\"\"";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::String(String::from("\\\"")), // Literal backslash then quote
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected, "Raw string with escaped quote failed");
}

#[test]
fn test_raw_string_double_backslash() {
    // r"\\"
    // c = \. next = \.
    // is_raw=true. next_char == quote? No (it is \).
    // else block:
    // push \.
    // if next_char == \ -> push \.
    // So result is \\
    let input = "r\"\\\\\"";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::String(String::from("\\\\")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected, "Raw string with double backslash failed");
}

#[test]
fn test_line_continuation_error() {
    // Expect error for backslash at end of line outside string
    let input = "\\\n";
    let tokens = lex(input);
    match &tokens[0] {
        TokenKind::Error(msg) => {
            assert!(msg.contains("Unexpected character"), "Message was: {}", msg)
        }
        _ => panic!(
            "Expected Error token for line continuation, got {:?}",
            tokens[0]
        ),
    }
}

#[test]
fn test_scientific_notation_complex() {
    // 1e+10 -> Integer(1), Identifier(e), Plus, Integer(10) ??
    // 1 -> Integer(1)
    // e -> Identifier start
    // + -> Plus
    // 10 -> Integer(10)
    // So: Integer(1), Identifier(e), Plus, Integer(10)
    let input = "1e+10";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::Integer(1),
        TokenKind::Identifier(String::from("e")),
        TokenKind::Plus,
        TokenKind::Integer(10),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected, "Scientific notation 1e+10 failed");
}

#[test]
fn test_complex_indentation_dedent_eof() {
    // Indent, then EOF. Should emit Dedent then EOF.
    let input = "a:\n  b";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::Identifier(String::from("a")),
        TokenKind::Colon,
        TokenKind::Newline,
        TokenKind::Indent,
        TokenKind::Identifier(String::from("b")),
        TokenKind::Newline,
        TokenKind::Dedent, // Auto-inserted at EOF
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected, "Indentation with EOF dedent failed");
}

#[test]
fn test_complex_indentation_multiple_dedent() {
    // a
    //   b
    //     c
    // d
    // Should be: a, NL, Indent, b, NL, Indent, c, NL, Dedent, Dedent, d, NL, EOF
    let input = "a\n  b\n    c\nd";
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
        TokenKind::Dedent,
        TokenKind::Identifier(String::from("d")),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected, "Multiple dedent failed");
}

#[test]
fn test_bytes_literal_escapes() {
    // b"\x00" -> should be handled?
    // Lexer::string calls current_literal.chars().map(|c| c as u8).collect()
    // But escapes are processed in string() loop.
    // '\' -> advance.
    // match escaped:
    // 'n' -> \n
    // etc.
    // It does NOT seem to handle \x hex escapes in `string` function!
    // It only handles n, t, r, \, ", ', newline.
    // So \x00 would be 'x', '0', '0' characters?
    // Let's test this behavior.
    let input = "b\"\\x00\"";
    let tokens = lex(input);
    // Expect: 'x', '0', '0' bytes because \x is not handled.
    // Wait, `\` then `x` -> default match `c => current_literal.push(c)`.
    // So it pushes 'x'. Then 0, 0.
    let expected = vec![
        TokenKind::Bytes(vec![b'x', b'0', b'0']),
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(
        tokens, expected,
        "Bytes literal with unhandled escape failed"
    );
}
