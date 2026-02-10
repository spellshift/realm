use eldritch_core::{Lexer, TokenKind};

fn lex(source: &str) -> Vec<TokenKind> {
    let mut lexer = Lexer::new(String::from(source));
    lexer.scan_tokens().into_iter().map(|t| t.kind).collect()
}

#[test]
fn test_fstring_nested_braces() {
    // f"{ {x} }" -> FStringContent containing [LBrace, Identifier(x), RBrace] (inside outer braces)
    // Wait, the outer braces are consumed by `tokenize_fstring_expression`.
    // The inner content is `{x}`.
    // So the tokens returned by `tokenize_fstring_expression` are LParen, tokens_of_expr, RParen.
    // The expression is `{x}` which is a Set containing x.
    // So tokens inside FStringContent should be: LParen, LBrace, Identifier(x), RBrace, RParen.
    let input = "f\"{ {x} }\"";
    let tokens = lex(input);

    if let TokenKind::FStringContent(inner_tokens) = &tokens[0] {
        let kinds: Vec<TokenKind> = inner_tokens.iter().map(|t| t.kind.clone()).collect();
        let expected = vec![
            TokenKind::LParen,
            TokenKind::LBrace,
            TokenKind::Identifier(String::from("x")),
            TokenKind::RBrace,
            TokenKind::RParen,
        ];
        assert_eq!(kinds, expected, "Nested braces failed");
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0]);
    }
}

#[test]
fn test_fstring_quote_containing_brace() {
    // f"{ '}' }" -> String containing '}'
    // Currently likely fails as '}' closes the expression early.
    let input = "f\"{ '}' }\"";
    let tokens = lex(input);

    if let TokenKind::FStringContent(inner_tokens) = &tokens[0] {
        let kinds: Vec<TokenKind> = inner_tokens.iter().map(|t| t.kind.clone()).collect();
        let expected = vec![
            TokenKind::LParen,
            TokenKind::String(String::from("}")), // The string literal contains "}"
            TokenKind::RParen,
        ];
        assert_eq!(kinds, expected, "Quote containing brace failed");
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0]);
    }
}

#[test]
fn test_fstring_double_quote_containing_brace() {
    // f'{ "}" }'
    let input = "f'{ \"}\" }'";
    let tokens = lex(input);

    if let TokenKind::FStringContent(inner_tokens) = &tokens[0] {
        let kinds: Vec<TokenKind> = inner_tokens.iter().map(|t| t.kind.clone()).collect();
        let expected = vec![
            TokenKind::LParen,
            TokenKind::String(String::from("}")),
            TokenKind::RParen,
        ];
        assert_eq!(kinds, expected, "Double quote containing brace failed");
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0]);
    }
}

#[test]
fn test_indentation_inside_parens() {
    // Indentation inside parentheses should be ignored (implicit line joining)
    let input = "(\n    x\n)";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::LParen,
        TokenKind::Identifier(String::from("x")),
        TokenKind::RParen,
        TokenKind::Newline,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_indentation_with_comment() {
    // Indentation with comment on the same line
    // "    # comment\n    pass" -> Indent, Newline, Pass, Dedent
    // The comment line counts as a line for indentation (emitting Indent), but then Newline.
    let input = "if True:\n    # comment\n    pass";
    let tokens = lex(input);
    let expected = vec![
        TokenKind::If,
        TokenKind::True,
        TokenKind::Colon,
        TokenKind::Newline,
        TokenKind::Indent,
        TokenKind::Newline, // from the comment line
        TokenKind::Pass,
        TokenKind::Newline,
        TokenKind::Dedent,
        TokenKind::Eof,
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_multiline_fstring_expression() {
    // Multiline expression inside f-string
    // f"""{ \n 1 + 1 \n }""" -> should work and ignore newlines inside expression
    let input = "f\"\"\"{ \n 1 + 1 \n }\"\"\"";
    let tokens = lex(input);

    if let TokenKind::FStringContent(inner_tokens) = &tokens[0] {
        let kinds: Vec<TokenKind> = inner_tokens.iter().map(|t| t.kind.clone()).collect();
        let expected = vec![
            TokenKind::LParen,
            TokenKind::Integer(1),
            TokenKind::Plus,
            TokenKind::Integer(1),
            TokenKind::RParen,
        ];
        assert_eq!(kinds, expected, "Multiline f-string expression failed");
    } else {
        panic!("Expected FStringContent, got {:?}", tokens[0]);
    }
}
