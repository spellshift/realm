extern crate std;

// TODO: Add lexer tests here!

use super::*;

fn lex(_input: &str) -> Vec<Token> {
    // TODO: Simple function to help Lex
    vec![]
}

// This test can be helpful to determine if we can run any tests at all.
#[test]
fn test_always_pass() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_empty_string() {
    assert_eq!(lex(""), vec![])
}
