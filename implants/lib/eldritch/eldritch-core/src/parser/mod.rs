use super::ast::Stmt;
use super::interpreter::error::{EldritchError, EldritchErrorKind};
use super::token::{Span, Token, TokenKind};
use alloc::vec::Vec;

pub mod expr;
pub mod stmt;

pub(crate) const MAX_PARSE_DEPTH: usize = 32;

pub struct Parser {
    pub(crate) tokens: Vec<Token>,
    pub(crate) current: usize,
    pub(crate) errors: Vec<EldritchError>,
    pub(crate) depth: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
            errors: Vec::new(),
            depth: 0,
        }
    }

    #[inline]
    pub(crate) fn enter_depth(&mut self) -> Result<(), EldritchError> {
        if self.depth >= MAX_PARSE_DEPTH {
            return self.error("Maximum nesting depth exceeded");
        }
        self.depth += 1;
        Ok(())
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

    pub(crate) fn error<T>(&self, msg: &str) -> Result<T, EldritchError> {
        let span = if self.current < self.tokens.len() {
            self.tokens[self.current].span
        } else {
            // Use last token span or dummy
            if let Some(last) = self.tokens.last() {
                last.span
            } else {
                Span::new(0, 0, 0)
            }
        };
        Err(EldritchError::new(
            EldritchErrorKind::SyntaxError,
            msg,
            span,
        ))
    }

    pub(crate) fn consume<F>(&mut self, check_fn: F, msg: &str) -> Result<&Token, EldritchError>
    where
        F: Fn(&TokenKind) -> bool,
    {
        if check_fn(&self.peek().kind) {
            Ok(self.advance())
        } else {
            self.error(msg)
        }
    }

    pub(crate) fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.tokens[self.current - 1].kind == TokenKind::Newline {
                return;
            }

            match self.peek().kind {
                TokenKind::Def
                | TokenKind::If
                | TokenKind::For
                | TokenKind::Try
                | TokenKind::Raise
                | TokenKind::Return
                | TokenKind::Pass
                | TokenKind::Break
                | TokenKind::Continue => return,
                _ => {}
            }

            self.advance();
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

    pub fn parse(&mut self) -> (Vec<Stmt>, Vec<EldritchError>) {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            while matches!(self.peek().kind, TokenKind::Newline) {
                self.advance();
            }
            if self.is_at_end() {
                break;
            }
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    self.errors.push(err.clone());
                    self.synchronize();
                    // Create an Error stmt to fill the gap
                    use super::ast::StmtKind;
                    let span = err.span;
                    statements.push(Stmt {
                        kind: StmtKind::Error(err.message),
                        span,
                    });
                }
            }
        }
        (statements, self.errors.clone())
    }

    #[allow(clippy::only_used_in_recursion)]
    pub(crate) fn validate_assignment_target(
        &self,
        expr: &super::ast::Expr,
    ) -> Result<(), EldritchError> {
        use super::ast::ExprKind;
        match &expr.kind {
            ExprKind::Identifier(_) => Ok(()),
            ExprKind::GetAttr(obj, _) => self.validate_assignment_target(obj),
            ExprKind::Index(obj, _) => self.validate_assignment_target(obj),
            ExprKind::Slice(obj, _, _, _) => self.validate_assignment_target(obj),
            ExprKind::Tuple(elements) | ExprKind::List(elements) => {
                for elem in elements {
                    self.validate_assignment_target(elem)?;
                }
                Ok(())
            }
            _ => Err(EldritchError::new(
                EldritchErrorKind::SyntaxError,
                "Invalid assignment target",
                expr.span,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use alloc::string::String;

    #[test]
    fn test_parser_depth_limit_parens() {
        let depth = MAX_PARSE_DEPTH + 20;
        let mut src = String::with_capacity(depth * 2 + 1);
        for _ in 0..depth {
            src.push('(');
        }
        src.push('1');
        for _ in 0..depth {
            src.push(')');
        }
        let tokens = Lexer::new(src).scan_tokens();
        let (_, errors) = Parser::new(tokens).parse();
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("Maximum nesting depth"));
    }

    #[test]
    fn test_parser_depth_limit_unary_and_not() {
        let depth = MAX_PARSE_DEPTH + 20;
        let mut src_not = String::new();
        for _ in 0..depth {
            src_not.push_str("not ");
        }
        src_not.push_str("True");
        let tokens = Lexer::new(src_not).scan_tokens();
        let (_, errors) = Parser::new(tokens).parse();
        assert!(!errors.is_empty());

        let mut src_neg = String::new();
        for _ in 0..depth {
            src_neg.push('-');
        }
        src_neg.push('1');
        let tokens = Lexer::new(src_neg).scan_tokens();
        let (_, errors) = Parser::new(tokens).parse();
        assert!(!errors.is_empty());
    }
}
