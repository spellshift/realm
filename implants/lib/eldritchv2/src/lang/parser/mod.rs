use super::token::{Token, TokenKind};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use super::ast::Stmt;

pub mod expr;
pub mod stmt;

pub struct Parser {
    pub(crate) tokens: Vec<Token>,
    pub(crate) current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub(crate) fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    pub(crate) fn peek_next(&self) -> &Token {
        if self.current + 1 < self.tokens.len() {
            &self.tokens[self.current + 1]
        } else {
            &self.tokens[self.current]
        }
    }

    pub(crate) fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        core::mem::discriminant(&self.peek().kind) == core::mem::discriminant(kind)
    }

    pub(crate) fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    pub(crate) fn consume<F>(&mut self, check_fn: F, msg: &str) -> Result<&Token, String>
    where
        F: Fn(&TokenKind) -> bool,
    {
        if check_fn(&self.peek().kind) {
            Ok(self.advance())
        } else {
            Err(msg.to_string())
        }
    }

    pub(crate) fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        for k in kinds {
            if core::mem::discriminant(&self.peek().kind) == core::mem::discriminant(k) {
                self.advance();
                return true;
            }
        }
        false
    }

    pub(crate) fn is_at_end(&self) -> bool {
        if self.current >= self.tokens.len() {
            return true;
        }
        matches!(self.tokens[self.current].kind, TokenKind::Eof)
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            while matches!(self.peek().kind, TokenKind::Newline) {
                self.advance();
            }
            if self.is_at_end() {
                break;
            }
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }
}
