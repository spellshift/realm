use super::super::ast::{ExprKind, Stmt, StmtKind};
use super::super::interpreter::error::EldritchError;
use super::super::token::{Span, TokenKind};
use super::Parser;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

impl Parser {
    pub(crate) fn make_stmt(&self, kind: StmtKind, start: Span, end: Span) -> Stmt {
        let span = Span::new(start.start, end.end, start.line);
        Stmt { kind, span }
    }

    pub(crate) fn declaration(&mut self) -> Result<Stmt, EldritchError> {
        if self.match_token(&[TokenKind::Def]) {
            self.function_def()
        } else {
            self.statement()
        }
    }

    fn function_def(&mut self) -> Result<Stmt, EldritchError> {
        let start_span = self.tokens[self.current - 1].span;

        // FIX: Extract name string immediately to drop borrow
        let name = {
            let token = self.consume(
                |t| matches!(t, TokenKind::Identifier(_)),
                "Expected function name.",
            )?;
            if let TokenKind::Identifier(s) = &token.kind {
                s.clone()
            } else {
                unreachable!()
            }
        };

        self.consume(
            |t| matches!(t, TokenKind::LParen),
            "Expected '(' after function name.",
        )?;

        let params = self.parse_function_params(TokenKind::RParen, true)?;

        self.consume(
            |t| matches!(t, TokenKind::RParen),
            "Expected ')' after parameters.",
        )?;

        let return_annotation = if self.match_token(&[TokenKind::Arrow]) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };

        self.consume(
            |t| matches!(t, TokenKind::Colon),
            "Expected ':' before function body.",
        )?;

        let body = self.parse_block_or_statement()?;
        let end_span = if let Some(last) = body.last() {
            last.span
        } else {
            start_span
        };

        Ok(self.make_stmt(
            StmtKind::Def(name, params, return_annotation, body),
            start_span,
            end_span,
        ))
    }

    pub(crate) fn parse_block_or_statement(&mut self) -> Result<Vec<Stmt>, EldritchError> {
        if self.match_token(&[TokenKind::Newline]) {
            self.consume(
                |t| matches!(t, TokenKind::Indent),
                "Expected indentation after newline.",
            )?;
            let mut stmts = Vec::new();
            while !self.check(&TokenKind::Dedent) && !self.is_at_end() {
                while matches!(self.peek().kind, TokenKind::Newline) {
                    self.advance();
                }
                if self.check(&TokenKind::Dedent) {
                    break;
                }
                stmts.push(self.declaration()?);
            }
            self.consume(
                |t| matches!(t, TokenKind::Dedent),
                "Expected dedent after block.",
            )?;
            Ok(stmts)
        } else {
            let stmt = self.statement()?;
            Ok(vec![stmt])
        }
    }

    fn statement(&mut self) -> Result<Stmt, EldritchError> {
        let start = self.peek().span;
        if self.match_token(&[TokenKind::If]) {
            self.if_statement(start)
        } else if self.match_token(&[TokenKind::Return]) {
            self.return_statement(start)
        } else if self.match_token(&[TokenKind::For]) {
            self.for_statement(start)
        } else if self.match_token(&[TokenKind::Break]) {
            if !self.is_at_end() && !matches!(self.peek().kind, TokenKind::Dedent) {
                self.consume(
                    |t| matches!(t, TokenKind::Newline),
                    "Expected newline after break.",
                )?;
            }
            Ok(self.make_stmt(StmtKind::Break, start, start))
        } else if self.match_token(&[TokenKind::Continue]) {
            if !self.is_at_end() && !matches!(self.peek().kind, TokenKind::Dedent) {
                self.consume(
                    |t| matches!(t, TokenKind::Newline),
                    "Expected newline after continue.",
                )?;
            }
            Ok(self.make_stmt(StmtKind::Continue, start, start))
        }
        // New: Handle pass statement
        else if self.match_token(&[TokenKind::Pass]) {
            if !self.is_at_end() && !matches!(self.peek().kind, TokenKind::Dedent) {
                self.consume(
                    |t| matches!(t, TokenKind::Newline),
                    "Expected newline after pass.",
                )?;
            }
            Ok(self.make_stmt(StmtKind::Pass, start, start))
        } else {
            self.assignment_or_expression_statement()
        }
    }

    fn for_statement(&mut self, start: Span) -> Result<Stmt, EldritchError> {
        let mut vars = Vec::new();
        loop {
            let ident_token = self.consume(
                |t| matches!(t, TokenKind::Identifier(_)),
                "Expected iteration variable name.",
            )?;
            let ident = match &ident_token.kind {
                TokenKind::Identifier(s) => s.clone(),
                _ => unreachable!(),
            };
            vars.push(ident);

            if self.match_token(&[TokenKind::Comma]) {
                continue;
            } else {
                break;
            }
        }

        self.consume(
            |t| matches!(t, TokenKind::In),
            "Expected 'in' after iteration variable.",
        )?;
        let iterable = self.expression()?;
        self.consume(
            |t| matches!(t, TokenKind::Colon),
            "Expected ':' before loop body.",
        )?;
        let body = self.parse_block_or_statement()?;
        let end = if let Some(last) = body.last() {
            last.span
        } else {
            iterable.span
        };
        Ok(self.make_stmt(StmtKind::For(vars, iterable, body), start, end))
    }

    fn if_statement(&mut self, start: Span) -> Result<Stmt, EldritchError> {
        let condition = self.expression()?;
        self.consume(
            |t| matches!(t, TokenKind::Colon),
            "Expected ':' after condition.",
        )?;
        let then_branch = self.parse_block_or_statement()?;
        let mut else_branch = None;
        let mut end = if let Some(last) = then_branch.last() {
            last.span
        } else {
            condition.span
        };

        if self.match_token(&[TokenKind::Elif]) {
            let inner_if = self.if_statement(self.tokens[self.current - 1].span)?;
            end = inner_if.span;
            else_branch = Some(vec![inner_if]);
        } else if self.match_token(&[TokenKind::Else]) {
            self.consume(
                |t| matches!(t, TokenKind::Colon),
                "Expected ':' after else.",
            )?;
            let else_stmts = self.parse_block_or_statement()?;
            if let Some(last) = else_stmts.last() {
                end = last.span;
            }
            else_branch = Some(else_stmts);
        }
        Ok(self.make_stmt(
            StmtKind::If(condition, then_branch, else_branch),
            start,
            end,
        ))
    }

    fn return_statement(&mut self, start: Span) -> Result<Stmt, EldritchError> {
        let mut value = None;
        let mut end = start;
        if !self.check(&TokenKind::Newline)
            && !self.check(&TokenKind::Eof)
            && !self.check(&TokenKind::Dedent)
        {
            let expr = self.expression()?;
            end = expr.span;
            value = Some(expr);
        }
        if !self.is_at_end() && !matches!(self.peek().kind, TokenKind::Dedent) {
            self.consume(
                |t| matches!(t, TokenKind::Newline),
                "Expected newline after return.",
            )?;
        }
        Ok(self.make_stmt(StmtKind::Return(value), start, end))
    }

    fn assignment_or_expression_statement(&mut self) -> Result<Stmt, EldritchError> {
        let mut expr = self.expression()?;
        let start = expr.span;

        // Handle tuple unpacking syntax: a, b = ...
        if self.match_token(&[TokenKind::Comma]) {
            let mut elements = vec![expr];

            // Allow trailing comma if next is Assign or Newline
            if !self.check(&TokenKind::Assign)
                && !self.check(&TokenKind::Newline)
                && !self.check(&TokenKind::Dedent)
            {
                loop {
                    elements.push(self.expression()?);
                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                    // Check if trailing comma
                    if self.check(&TokenKind::Assign) || self.check(&TokenKind::Newline) {
                        break;
                    }
                }
            }

            let end_span = elements.last().unwrap().span;
            expr = self.make_expr(ExprKind::Tuple(elements), start, end_span);
        }

        let mut end = expr.span;

        // Check for annotated assignment: x: int = 5
        // Note: We only support annotated assignment if it is followed by '='
        // x: int by itself is a valid statement in Python but the user said "no" to `x: int`, only `x: int = 10`.
        // Wait, the user said `x: int` is NOT valid. `x: int = 10` IS valid.
        // So I must enforce '=' if I see ':'.

        let annotation = if self.match_token(&[TokenKind::Colon]) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };

        // Normal assignment
        if self.match_token(&[TokenKind::Assign]) {
            self.validate_assignment_target(&expr)?;
            let value = self.expression()?;

            // Allow explicit tuple on RHS too: a, b = 1, 2
            let mut final_value = value;
            if self.match_token(&[TokenKind::Comma]) {
                let mut elements = vec![final_value];
                if !self.check(&TokenKind::Newline) && !self.check(&TokenKind::Dedent) {
                    loop {
                        elements.push(self.expression()?);
                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                        if self.check(&TokenKind::Newline) {
                            break;
                        }
                    }
                }
                let end_span = elements.last().unwrap().span;
                let start_span = elements[0].span;
                final_value = self.make_expr(ExprKind::Tuple(elements), start_span, end_span);
            }

            end = final_value.span;
            if !self.is_at_end() && !matches!(self.peek().kind, TokenKind::Dedent) {
                self.consume(
                    |t| matches!(t, TokenKind::Newline),
                    "Expected newline after assignment.",
                )?;
            }
            return Ok(self.make_stmt(
                StmtKind::Assignment(expr, annotation, final_value),
                start,
                end,
            ));
        } else if annotation.is_some() {
            return self.error("Annotated variable must be assigned a value.");
        }

        // Augmented assignment
        if self.match_token(&[
            TokenKind::PlusAssign,
            TokenKind::MinusAssign,
            TokenKind::StarAssign,
            TokenKind::SlashAssign,
            TokenKind::SlashSlashAssign,
            TokenKind::PercentAssign,
        ]) {
            self.validate_assignment_target(&expr)?;

            // Augmented assignment does not support tuple unpacking
            if let ExprKind::Tuple(_) = expr.kind {
                return self.error("Augmented assignment does not support tuple unpacking");
            }
            if let ExprKind::List(_) = expr.kind {
                return self.error("Augmented assignment does not support list unpacking");
            }

            let op = self.tokens[self.current - 1].kind.clone();
            let value = self.expression()?;
            end = value.span;
            if !self.is_at_end() && !matches!(self.peek().kind, TokenKind::Dedent) {
                self.consume(
                    |t| matches!(t, TokenKind::Newline),
                    "Expected newline after augmented assignment.",
                )?;
            }
            return Ok(self.make_stmt(StmtKind::AugmentedAssignment(expr, op, value), start, end));
        }

        if !self.is_at_end() && !matches!(self.peek().kind, TokenKind::Dedent) {
            self.consume(
                |t| matches!(t, TokenKind::Newline),
                "Expected newline after expression.",
            )?;
        }
        Ok(self.make_stmt(StmtKind::Expression(expr), start, end))
    }
}
