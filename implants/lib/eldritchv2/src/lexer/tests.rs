extern crate std;
use alloc::string::ToString;
use super::*;
fn lex(input: &str) -> Vec<Token> {
    Lexer::new(input).filter(|t| t != &Token::Newline).collect()
}
#[test]
fn test_snippet() {
    let input = r#"
// A user-defined function with parameters!
def a_user_defined_func(param1, param2):
    // A loop!
    for i in range 10:
        // A built-in function!
        print(i)
        // A nested-loop!
        for i in range 20:
            print(i)
    // A conditional
    if param1 > param2:
        return true
    elif param1 < param2:
        return false
    else:
        // they're equal!
        return false
"#;
    let tokens = lex(input);
    let expected_tokens = vec![
        Token::Def,
        Token::Identifier("a_user_defined_func".to_string()),
        Token::LParen,
        Token::Identifier("param1".to_string()),
        Token::Comma,
        Token::Identifier("param2".to_string()),
        Token::RParen,
        Token::Colon,
        Token::Indent,
        Token::For,
        Token::Identifier("i".to_string()),
        Token::In,
        Token::Identifier("range".to_string()),
        Token::Int(10),
        Token::Colon,
        Token::Indent,
        Token::Identifier("print".to_string()),
        Token::LParen,
        Token::Identifier("i".to_string()),
        Token::RParen,
        Token::For,
        Token::Identifier("i".to_string()),
        Token::In,
        Token::Identifier("range".to_string()),
        Token::Int(20),
        Token::Colon,
        Token::Indent,
        Token::Identifier("print".to_string()),
        Token::LParen,
        Token::Identifier("i".to_string()),
        Token::RParen,
        Token::Outdent,
        Token::Outdent,
        Token::If,
        Token::Identifier("param1".to_string()),
        Token::Greater,
        Token::Identifier("param2".to_string()),
        Token::Colon,
        Token::Indent,
        Token::Return,
        Token::Identifier("true".to_string()),
        Token::Outdent,
        Token::Elif,
        Token::Identifier("param1".to_string()),
        Token::Less,
        Token::Identifier("param2".to_string()),
        Token::Colon,
        Token::Indent,
        Token::Return,
        Token::Identifier("false".to_string()),
        Token::Outdent,
        Token::Else,
        Token::Colon,
        Token::Indent,
        Token::Return,
        Token::Identifier("false".to_string()),
        Token::Outdent,
        Token::Outdent,
    ];
    assert_eq!(tokens, expected_tokens);
}
#[test]
fn test_simple_indentation() {
    let input = "
if true:
    return 1
";
    let tokens = lex(input);
    let expected = vec![
        Token::If,
        Token::Identifier("true".to_string()),
        Token::Colon,
        Token::Indent,
        Token::Return,
        Token::Int(1),
        Token::Outdent,
    ];
    assert_eq!(tokens, expected);
}
#[test]
fn test_mixed_indentation() {
    let input = "
def f():
  pass
	pass
";
    let tokens = lex(input);
    let expected = vec![
        Token::Def,
        Token::Identifier("f".to_string()),
        Token::LParen,
        Token::RParen,
        Token::Colon,
        Token::Indent,
        Token::Identifier("pass".to_string()),
        Token::Indent,
        Token::Identifier("pass".to_string()),
        Token::Outdent,
        Token::Outdent,
    ];
    assert_eq!(tokens, expected);
}
