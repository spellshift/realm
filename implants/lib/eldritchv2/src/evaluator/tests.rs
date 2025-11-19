use super::*;
use crate::lexer::Lexer;
use crate::parser::Parser;

fn eval(input: &str) -> Option<Object> {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    let mut evaluator = Evaluator::new();
    evaluator.eval_program(&program)
}

#[test]
fn test_eval_integer_expression() {
    let result = eval("5\n").unwrap();
    assert_eq!(result, Object::Integer(5));
}

#[test]
fn test_eval_return_statement() {
    let result = eval("return 5\n").unwrap();
    assert_eq!(result, Object::Integer(5));
}

#[test]
fn test_eval_if_statement() {
    let result = eval("if true:\n    return 5\n").unwrap();
    assert_eq!(result, Object::Integer(5));
}
