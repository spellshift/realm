use crate::lexer::Token;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Statement(Statement),
    Expression(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Assign(String, Expression),
    Return(Expression),
    Expression(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Identifier(String),
    IntegerLiteral(i64),
    StringLiteral(String),
    Prefix(Token, Box<Expression>),
    Infix(Token, Box<Expression>, Box<Expression>),
    Boolean(bool),
    If {
        condition: Box<Expression>,
        consequence: BlockStatement,
        alternative: Option<BlockStatement>,
    },
    FunctionLiteral {
        parameters: Vec<String>,
        body: BlockStatement,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
}
