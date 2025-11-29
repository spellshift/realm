use super::ast::{Argument, Expr, ExprKind, FStringSegment, Param, Stmt, StmtKind, Value};
use super::token::{Span, Token, TokenKind};
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    fn peek_next(&self) -> &Token {
        if self.current + 1 < self.tokens.len() {
            &self.tokens[self.current + 1]
        } else {
            &self.tokens[self.current]
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        core::mem::discriminant(&self.peek().kind) == core::mem::discriminant(kind)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    fn consume<F>(&mut self, check_fn: F, msg: &str) -> Result<&Token, String>
    where
        F: Fn(&TokenKind) -> bool,
    {
        if check_fn(&self.peek().kind) {
            Ok(self.advance())
        } else {
            Err(msg.to_string())
        }
    }

    fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        for k in kinds {
            if core::mem::discriminant(&self.peek().kind) == core::mem::discriminant(k) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn is_at_end(&self) -> bool {
        if self.current >= self.tokens.len() {
            return true;
        }
        matches!(self.tokens[self.current].kind, TokenKind::Eof)
    }

    // Helper to create Expr with span
    fn make_expr(&self, kind: ExprKind, start: Span, end: Span) -> Expr {
        let span = Span::new(start.start, end.end, start.line);
        Expr { kind, span }
    }

    fn make_stmt(&self, kind: StmtKind, start: Span, end: Span) -> Stmt {
        let span = Span::new(start.start, end.end, start.line);
        Stmt { kind, span }
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

    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.match_token(&[TokenKind::Def]) {
            self.function_def()
        } else {
            self.statement()
        }
    }

    fn function_def(&mut self) -> Result<Stmt, String> {
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

        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                if self.match_token(&[TokenKind::Star]) {
                    let param_token = self.consume(
                        |t| matches!(t, TokenKind::Identifier(_)),
                        "Expected name after *.",
                    )?;
                    if let TokenKind::Identifier(param_name) = &param_token.kind {
                        params.push(Param::Star(param_name.clone()));
                    }
                } else if self.match_token(&[TokenKind::StarStar]) {
                    let param_token = self.consume(
                        |t| matches!(t, TokenKind::Identifier(_)),
                        "Expected name after **.",
                    )?;
                    if let TokenKind::Identifier(param_name) = &param_token.kind {
                        params.push(Param::StarStar(param_name.clone()));
                    }
                } else {
                    // FIX: Separate consume from logic to avoid multi-borrow
                    let param_name = {
                        let token = self.consume(
                            |t| matches!(t, TokenKind::Identifier(_)),
                            "Expected parameter name.",
                        )?;
                        if let TokenKind::Identifier(name) = &token.kind {
                            name.clone()
                        } else {
                            unreachable!()
                        }
                    };

                    if self.match_token(&[TokenKind::Assign]) {
                        let default_val = self.expression()?;
                        params.push(Param::WithDefault(param_name, default_val));
                    } else {
                        params.push(Param::Normal(param_name));
                    }
                }

                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.consume(
            |t| matches!(t, TokenKind::RParen),
            "Expected ')' after parameters.",
        )?;
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

        Ok(self.make_stmt(StmtKind::Def(name, params, body), start_span, end_span))
    }

    fn parse_block_or_statement(&mut self) -> Result<Vec<Stmt>, String> {
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

    fn statement(&mut self) -> Result<Stmt, String> {
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
            self.expression_statement()
        }
    }

    // ... (rest of the file unchanged)
    fn for_statement(&mut self, start: Span) -> Result<Stmt, String> {
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

    fn if_statement(&mut self, start: Span) -> Result<Stmt, String> {
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

    fn return_statement(&mut self, start: Span) -> Result<Stmt, String> {
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

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        let start = expr.span;
        let mut end = expr.span;

        if self.match_token(&[TokenKind::Assign]) {
            if let ExprKind::Identifier(name) = expr.kind {
                let value = self.expression()?;
                end = value.span;
                if !self.is_at_end() && !matches!(self.peek().kind, TokenKind::Dedent) {
                    self.consume(
                        |t| matches!(t, TokenKind::Newline),
                        "Expected newline after assignment.",
                    )?;
                }
                return Ok(self.make_stmt(StmtKind::Assignment(name, value), start, end));
            } else {
                return Err("Invalid assignment target.".to_string());
            }
        }
        if !self.is_at_end() && !matches!(self.peek().kind, TokenKind::Dedent) {
            self.consume(
                |t| matches!(t, TokenKind::Newline),
                "Expected newline after expression.",
            )?;
        }
        Ok(self.make_stmt(StmtKind::Expression(expr), start, end))
    }

    fn expression(&mut self) -> Result<Expr, String> {
        if self.match_token(&[TokenKind::Lambda]) {
            return self.lambda_expression();
        }
        self.logic_or()
    }

    fn lambda_expression(&mut self) -> Result<Expr, String> {
        let start_span = self.tokens[self.current - 1].span;
        let params = self.parse_function_params(TokenKind::Colon)?;
        self.consume(
            |t| matches!(t, TokenKind::Colon),
            "Expected ':' after lambda params.",
        )?;
        let body = self.expression()?;
        let end_span = body.span;
        Ok(self.make_expr(
            ExprKind::Lambda {
                params,
                body: Box::new(body),
            },
            start_span,
            end_span,
        ))
    }

    // Helper to parse params for both def and lambda
    fn parse_function_params(&mut self, terminator: TokenKind) -> Result<Vec<Param>, String> {
        let mut params = Vec::new();
        if !self.check(&terminator) {
            loop {
                if self.match_token(&[TokenKind::Star]) {
                    let param_token = self.consume(
                        |t| matches!(t, TokenKind::Identifier(_)),
                        "Expected name after *.",
                    )?;
                    if let TokenKind::Identifier(param_name) = &param_token.kind {
                        params.push(Param::Star(param_name.clone()));
                    }
                } else if self.match_token(&[TokenKind::StarStar]) {
                    let param_token = self.consume(
                        |t| matches!(t, TokenKind::Identifier(_)),
                        "Expected name after **.",
                    )?;
                    if let TokenKind::Identifier(param_name) = &param_token.kind {
                        params.push(Param::StarStar(param_name.clone()));
                    }
                } else {
                    let param_name = {
                        let token = self.consume(
                            |t| matches!(t, TokenKind::Identifier(_)),
                            "Expected parameter name.",
                        )?;
                        if let TokenKind::Identifier(name) = &token.kind {
                            name.clone()
                        } else {
                            unreachable!()
                        }
                    };

                    if self.match_token(&[TokenKind::Assign]) {
                        let default_val = self.expression()?;
                        params.push(Param::WithDefault(param_name, default_val));
                    } else {
                        params.push(Param::Normal(param_name));
                    }
                }

                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        Ok(params)
    }

    fn logic_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.logic_and()?;
        while self.match_token(&[TokenKind::Or]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.logic_and()?;
            let start = expr.span;
            let end = right.span;
            expr = self.make_expr(
                ExprKind::LogicalOp(Box::new(expr), operator.kind, Box::new(right)),
                start,
                end,
            );
        }
        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.logic_not()?;
        while self.match_token(&[TokenKind::And]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.logic_not()?;
            let start = expr.span;
            let end = right.span;
            expr = self.make_expr(
                ExprKind::LogicalOp(Box::new(expr), operator.kind, Box::new(right)),
                start,
                end,
            );
        }
        Ok(expr)
    }

    fn logic_not(&mut self) -> Result<Expr, String> {
        if self.match_token(&[TokenKind::Not]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.logic_not()?;
            let start = operator.span;
            let end = right.span;
            return Ok(self.make_expr(
                ExprKind::UnaryOp(operator.kind, Box::new(right)),
                start,
                end,
            ));
        }
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;
        while self.match_token(&[TokenKind::Eq, TokenKind::NotEq]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.comparison()?;
            let start = expr.span;
            let end = right.span;
            expr = self.make_expr(
                ExprKind::BinaryOp(Box::new(expr), operator.kind, Box::new(right)),
                start,
                end,
            );
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.bitwise_or()?;
        while self.match_token(&[
            TokenKind::Lt,
            TokenKind::Gt,
            TokenKind::LtEq,
            TokenKind::GtEq,
        ]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.bitwise_or()?;
            let start = expr.span;
            let end = right.span;
            expr = self.make_expr(
                ExprKind::BinaryOp(Box::new(expr), operator.kind, Box::new(right)),
                start,
                end,
            );
        }
        Ok(expr)
    }

    fn bitwise_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.bitwise_xor()?;
        while self.match_token(&[TokenKind::BitOr]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.bitwise_xor()?;
            let start = expr.span;
            let end = right.span;
            expr = self.make_expr(
                ExprKind::BinaryOp(Box::new(expr), operator.kind, Box::new(right)),
                start,
                end,
            );
        }
        Ok(expr)
    }

    fn bitwise_xor(&mut self) -> Result<Expr, String> {
        let mut expr = self.bitwise_and()?;
        while self.match_token(&[TokenKind::BitXor]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.bitwise_and()?;
            let start = expr.span;
            let end = right.span;
            expr = self.make_expr(
                ExprKind::BinaryOp(Box::new(expr), operator.kind, Box::new(right)),
                start,
                end,
            );
        }
        Ok(expr)
    }

    fn bitwise_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.shift()?;
        while self.match_token(&[TokenKind::BitAnd]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.shift()?;
            let start = expr.span;
            let end = right.span;
            expr = self.make_expr(
                ExprKind::BinaryOp(Box::new(expr), operator.kind, Box::new(right)),
                start,
                end,
            );
        }
        Ok(expr)
    }

    fn shift(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;
        while self.match_token(&[TokenKind::LShift, TokenKind::RShift]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.term()?;
            let start = expr.span;
            let end = right.span;
            expr = self.make_expr(
                ExprKind::BinaryOp(Box::new(expr), operator.kind, Box::new(right)),
                start,
                end,
            );
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;
        while self.match_token(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.factor()?;
            let start = expr.span;
            let end = right.span;
            expr = self.make_expr(
                ExprKind::BinaryOp(Box::new(expr), operator.kind, Box::new(right)),
                start,
                end,
            );
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        if self.match_token(&[TokenKind::Minus, TokenKind::BitNot]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.factor()?;
            let start = operator.span;
            let end = right.span;
            return Ok(self.make_expr(
                ExprKind::UnaryOp(operator.kind, Box::new(right)),
                start,
                end,
            ));
        }

        let mut expr = self.call()?;
        while self.match_token(&[TokenKind::Slash, TokenKind::Star, TokenKind::Percent]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.call()?;
            let start = expr.span;
            let end = right.span;
            expr = self.make_expr(
                ExprKind::BinaryOp(Box::new(expr), operator.kind, Box::new(right)),
                start,
                end,
            );
        }
        Ok(expr)
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenKind::LParen]) {
                let _start = expr.span;
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[TokenKind::LBracket]) {
                let start_span = expr.span;
                let mut start = None;
                let mut stop = None;
                let mut step = None;
                let mut is_slice = false;

                if self.match_token(&[TokenKind::Colon]) {
                    is_slice = true;
                    if !self.check(&TokenKind::Colon) && !self.check(&TokenKind::RBracket) {
                        stop = Some(Box::new(self.expression()?));
                    }
                    if self.match_token(&[TokenKind::Colon]) {
                        if !self.check(&TokenKind::RBracket) {
                            step = Some(Box::new(self.expression()?));
                        }
                    }
                } else {
                    start = Some(Box::new(self.expression()?));
                    if self.match_token(&[TokenKind::Colon]) {
                        is_slice = true;
                        if !self.check(&TokenKind::Colon) && !self.check(&TokenKind::RBracket) {
                            stop = Some(Box::new(self.expression()?));
                        }
                        if self.match_token(&[TokenKind::Colon]) {
                            if !self.check(&TokenKind::RBracket) {
                                step = Some(Box::new(self.expression()?));
                            }
                        }
                    }
                }

                // FIX: Extract span to drop borrow of self from consume
                let end_span = self
                    .consume(
                        |t| matches!(t, TokenKind::RBracket),
                        "Expected ']' after subscript.",
                    )?
                    .span;

                if is_slice {
                    expr = self.make_expr(
                        ExprKind::Slice(Box::new(expr), start, stop, step),
                        start_span,
                        end_span,
                    );
                } else {
                    expr = self.make_expr(
                        ExprKind::Index(Box::new(expr), start.unwrap()),
                        start_span,
                        end_span,
                    );
                }
            } else if self.match_token(&[TokenKind::Dot]) {
                let start = expr.span;
                // FIX: Extract name and span to drop borrow
                let (name, end_span) = {
                    let token = self.consume(
                        |t| matches!(t, TokenKind::Identifier(_)),
                        "Expect property name after '.'.",
                    )?;
                    let n = if let TokenKind::Identifier(name) = &token.kind {
                        name.clone()
                    } else {
                        unreachable!()
                    };
                    (n, token.span)
                };
                expr = self.make_expr(ExprKind::GetAttr(Box::new(expr), name), start, end_span);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, String> {
        let start = callee.span;
        let mut args = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                let is_keyword = if let TokenKind::Identifier(_) = self.peek().kind {
                    matches!(self.peek_next().kind, TokenKind::Assign)
                } else {
                    false
                };

                if self.match_token(&[TokenKind::Star]) {
                    let expr = self.expression()?;
                    args.push(Argument::StarArgs(expr));
                } else if self.match_token(&[TokenKind::StarStar]) {
                    let expr = self.expression()?;
                    args.push(Argument::KwArgs(expr));
                } else if is_keyword {
                    // FIX: Clone to avoid borrow issues, ensure advance is done
                    let name_token = self.advance();
                    let name = if let TokenKind::Identifier(s) = &name_token.kind {
                        s.clone()
                    } else {
                        unreachable!()
                    };
                    self.advance(); // Consume '='
                    let val = self.expression()?;
                    args.push(Argument::Keyword(name, val));
                } else {
                    args.push(Argument::Positional(self.expression()?));
                }

                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        let end_span = self
            .consume(
                |t| matches!(t, TokenKind::RParen),
                "Expected ')' after arguments.",
            )?
            .span;
        Ok(self.make_expr(ExprKind::Call(Box::new(callee), args), start, end_span))
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.peek().clone();
        let span = token.span;

        if self.match_token(&[TokenKind::False]) {
            return Ok(self.make_expr(ExprKind::Literal(Value::Bool(false)), span, span));
        }
        if self.match_token(&[TokenKind::True]) {
            return Ok(self.make_expr(ExprKind::Literal(Value::Bool(true)), span, span));
        }
        if self.match_token(&[TokenKind::None]) {
            return Ok(self.make_expr(ExprKind::Literal(Value::None), span, span));
        }

        if let TokenKind::Integer(i) = token.kind {
            self.advance();
            return Ok(self.make_expr(ExprKind::Literal(Value::Int(i)), span, span));
        }

        if let TokenKind::String(s) = &token.kind {
            self.advance();
            return Ok(self.make_expr(ExprKind::Literal(Value::String(s.clone())), span, span));
        }

        if let TokenKind::Bytes(b) = &token.kind {
            self.advance();
            return Ok(self.make_expr(ExprKind::Literal(Value::Bytes(b.clone())), span, span));
        }

        if let TokenKind::FStringContent(fstring_tokens) = &token.kind {
            self.advance();
            return self.parse_fstring_content(fstring_tokens.clone(), span);
        }

        if let TokenKind::Identifier(s) = &token.kind {
            self.advance();
            return Ok(self.make_expr(ExprKind::Identifier(s.clone()), span, span));
        }

        if self.match_token(&[TokenKind::LParen]) {
            if self.match_token(&[TokenKind::RParen]) {
                let end = self.tokens[self.current - 1].span;
                return Ok(self.make_expr(ExprKind::Tuple(Vec::new()), span, end));
            }
            let expr = self.expression()?;
            if self.match_token(&[TokenKind::Comma]) {
                let mut elements = vec![expr];
                if !self.check(&TokenKind::RParen) {
                    loop {
                        elements.push(self.expression()?);
                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }
                let end = self
                    .consume(
                        |t| matches!(t, TokenKind::RParen),
                        "Expected ')' after tuple.",
                    )?
                    .span;
                return Ok(self.make_expr(ExprKind::Tuple(elements), span, end));
            }
            let _end = self
                .consume(
                    |t| matches!(t, TokenKind::RParen),
                    "Expected ')' after expression.",
                )?
                .span; // Fixed unused variable
            return Ok(expr);
        }

        if self.match_token(&[TokenKind::LBracket]) {
            if self.match_token(&[TokenKind::RBracket]) {
                let end = self.tokens[self.current - 1].span;
                return Ok(self.make_expr(ExprKind::List(Vec::new()), span, end));
            }

            let first_expr = self.expression()?;

            if self.match_token(&[TokenKind::For]) {
                let (var, _) = {
                    let t = self.consume(
                        |t| matches!(t, TokenKind::Identifier(_)),
                        "Expected iteration variable.",
                    )?;
                    let v = if let TokenKind::Identifier(s) = &t.kind {
                        s.clone()
                    } else {
                        unreachable!()
                    };
                    (v, t.span)
                };

                self.consume(|t| matches!(t, TokenKind::In), "Expected 'in'.")?;
                let iterable = self.expression()?;
                let mut cond = None;
                if self.match_token(&[TokenKind::If]) {
                    cond = Some(Box::new(self.expression()?));
                }
                let end = self
                    .consume(|t| matches!(t, TokenKind::RBracket), "Expected ']'.")?
                    .span;
                return Ok(self.make_expr(
                    ExprKind::ListComp {
                        body: Box::new(first_expr),
                        var,
                        iterable: Box::new(iterable),
                        cond,
                    },
                    span,
                    end,
                ));
            }

            let mut elements = vec![first_expr];
            if self.match_token(&[TokenKind::Comma]) {
                if !self.check(&TokenKind::RBracket) {
                    loop {
                        elements.push(self.expression()?);
                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }
            }
            let end = self
                .consume(
                    |t| matches!(t, TokenKind::RBracket),
                    "Expected ']' after list.",
                )?
                .span;
            return Ok(self.make_expr(ExprKind::List(elements), span, end));
        }

        if self.match_token(&[TokenKind::LBrace]) {
            if self.match_token(&[TokenKind::RBrace]) {
                let end = self.tokens[self.current - 1].span;
                return Ok(self.make_expr(ExprKind::Dictionary(Vec::new()), span, end));
            }

            let key_expr = self.expression()?;
            self.consume(|t| matches!(t, TokenKind::Colon), "Expected ':' after key.")?;
            let val_expr = self.expression()?;

            if self.match_token(&[TokenKind::For]) {
                let (var, _) = {
                    let t = self.consume(
                        |t| matches!(t, TokenKind::Identifier(_)),
                        "Expected iteration variable.",
                    )?;
                    let v = if let TokenKind::Identifier(s) = &t.kind {
                        s.clone()
                    } else {
                        unreachable!()
                    };
                    (v, t.span)
                };

                self.consume(|t| matches!(t, TokenKind::In), "Expected 'in'.")?;
                let iterable = self.expression()?;
                let mut cond = None;
                if self.match_token(&[TokenKind::If]) {
                    cond = Some(Box::new(self.expression()?));
                }
                let end = self
                    .consume(|t| matches!(t, TokenKind::RBrace), "Expected '}'.")?
                    .span;
                return Ok(self.make_expr(
                    ExprKind::DictComp {
                        key: Box::new(key_expr),
                        value: Box::new(val_expr),
                        var,
                        iterable: Box::new(iterable),
                        cond,
                    },
                    span,
                    end,
                ));
            }

            let mut entries = vec![(key_expr, val_expr)];
            if self.match_token(&[TokenKind::Comma]) {
                if !self.check(&TokenKind::RBrace) {
                    loop {
                        let k = self.expression()?;
                        self.consume(|t| matches!(t, TokenKind::Colon), "Expected ':'.")?;
                        let v = self.expression()?;
                        entries.push((k, v));
                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }
            }
            let end = self
                .consume(
                    |t| matches!(t, TokenKind::RBrace),
                    "Expected '}' after dictionary definition.",
                )?
                .span;
            return Ok(self.make_expr(ExprKind::Dictionary(entries), span, end));
        }

        Err(format!("Expect expression. Found {:?}", self.peek()))
    }

    fn parse_fstring_content(
        &mut self,
        fstring_tokens: Vec<Token>,
        start_span: Span,
    ) -> Result<Expr, String> {
        let mut tokens_with_eof = fstring_tokens;
        // Use start_span to create a dummy EOF if needed, though FStringContent tokens should have valid spans
        let eof_span = if let Some(last) = tokens_with_eof.last() {
            last.span
        } else {
            start_span
        };
        tokens_with_eof.push(Token {
            kind: TokenKind::Eof,
            span: eof_span,
        });

        let mut internal_parser = Parser::new(tokens_with_eof);
        let mut segments = Vec::new();

        while !internal_parser.is_at_end() {
            if let TokenKind::String(s) = &internal_parser.peek().kind {
                segments.push(FStringSegment::Literal(s.clone()));
                internal_parser.advance();
            } else if internal_parser.match_token(&[TokenKind::LParen]) {
                let expr = internal_parser.expression()?;
                segments.push(FStringSegment::Expression(expr));
                internal_parser.consume(
                    |t| matches!(t, TokenKind::RParen),
                    "Expected ')' to close f-string embedded expression.",
                )?;
            } else {
                return Err(format!(
                    "Unexpected token in f-string content: {:?}",
                    internal_parser.peek()
                ));
            }
        }
        // Use start_span as rough location
        Ok(self.make_expr(ExprKind::FString(segments), start_span, eof_span))
    }
}
