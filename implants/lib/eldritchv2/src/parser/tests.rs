use super::*;
use crate::ast::{Expression, Operand, PrimaryExpr, SmallStmt, Statement, Suite};
use crate::lexer::Lexer;
use alloc::string::ToString;
use alloc::vec;

#[test]
fn test_parse_integer_expression() {
    let input = "5\n";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    assert_eq!(program.statements.len(), 1);

    let expected = Statement::Simple(vec![SmallStmt::Expr(vec![Expression::Primary(
        PrimaryExpr::Operand(Operand::Int(5)),
    )])]);

    assert_eq!(program.statements[0], expected);
}

#[test]
fn test_parse_return_statement() {
    let input = "return 5\n";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    assert_eq!(program.statements.len(), 1);

    let expected = Statement::Simple(vec![SmallStmt::Return(Some(vec![Expression::Primary(
        PrimaryExpr::Operand(Operand::Int(5)),
    )]))]);

    assert_eq!(program.statements[0], expected);
}

#[test]
fn test_parse_if_statement() {
    let input = "if true:\n    return 5\n";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    assert_eq!(program.statements.len(), 1);

    let expected = Statement::If {
        condition: Expression::Primary(PrimaryExpr::Operand(Operand::Identifier(
            "true".to_string(),
        ))),
        consequence: Suite {
            statements: vec![Statement::Simple(vec![SmallStmt::Return(Some(vec![
                Expression::Primary(PrimaryExpr::Operand(Operand::Int(5))),
            ]))])],
        },
        alternatives: vec![],
        default: None,
    };

    assert_eq!(program.statements[0], expected);
}
