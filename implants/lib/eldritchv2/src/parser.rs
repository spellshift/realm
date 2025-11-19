use crate::ast::{Expression, Operand, PrimaryExpr, Program, SmallStmt, Statement, Suite};
use crate::lexer::{Lexer, Token};
use alloc::vec;
use alloc::vec::Vec;
use core::iter::Peekable;

pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Parser {
            lexer: lexer.peekable(),
        }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program {
            statements: Vec::new(),
        };

        while self.peek_token().is_some() && self.peek_token() != Some(&Token::Eof) {
            if let Some(statement) = self.parse_statement() {
                program.statements.push(statement);
            } else {
                break;
            }
        }

        program
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.peek_token() {
            Some(Token::If) => self.parse_if_statement(),
            _ => {
                let stmt = self.parse_small_statement()?;
                if self.peek_token() == Some(&Token::Newline) {
                    self.next_token();
                }
                Some(Statement::Simple(vec![stmt]))
            }
        }
    }

    fn parse_if_statement(&mut self) -> Option<Statement> {
        self.next_token(); // Consume 'if'
        let condition = self.parse_expression()?;
        self.expect_peek(&Token::Colon);
        self.next_token();
        let consequence = self.parse_suite()?;
        Some(Statement::If {
            condition,
            consequence,
            alternatives: vec![],
            default: None,
        })
    }

    fn parse_suite(&mut self) -> Option<Suite> {
        self.expect_peek(&Token::Newline);
        self.next_token();
        self.expect_peek(&Token::Indent);
        self.next_token();

        let mut statements = Vec::new();
        while self.peek_token() != Some(&Token::Outdent) {
            statements.push(self.parse_statement()?);
        }

        self.expect_peek(&Token::Outdent);
        self.next_token();

        Some(Suite { statements })
    }

    fn parse_small_statement(&mut self) -> Option<SmallStmt> {
        match self.peek_token() {
            Some(Token::Return) => self.parse_return_statement(),
            _ => {
                let expr = self.parse_expression()?;
                Some(SmallStmt::Expr(vec![expr]))
            }
        }
    }

    fn parse_return_statement(&mut self) -> Option<SmallStmt> {
        self.next_token(); // Consume 'return'
        let expr = self.parse_expression()?;
        Some(SmallStmt::Return(Some(vec![expr])))
    }

    fn parse_expression(&mut self) -> Option<Expression> {
        self.parse_primary_expression()
    }

    fn parse_primary_expression(&mut self) -> Option<Expression> {
        Some(Expression::Primary(PrimaryExpr::Operand(
            self.parse_operand()?,
        )))
    }

    fn parse_operand(&mut self) -> Option<Operand> {
        match self.next_token() {
            Some(Token::Int(value)) => Some(Operand::Int(value)),
            Some(Token::Identifier(name)) => Some(Operand::Identifier(name)),
            _ => None,
        }
    }

    fn peek_token(&mut self) -> Option<&Token> {
        self.lexer.peek()
    }

    fn next_token(&mut self) -> Option<Token> {
        self.lexer.next()
    }

    fn expect_peek(&mut self, expected: &Token) {
        if self.peek_token() != Some(expected) {
            // A simple error handling. In a real parser, you'd want to
            // create a proper error type and return a Result.
            panic!(
                "Expected to see token {:?}, but found {:?}",
                expected,
                self.peek_token()
            );
        }
    }
}

#[cfg(test)]
mod tests;
