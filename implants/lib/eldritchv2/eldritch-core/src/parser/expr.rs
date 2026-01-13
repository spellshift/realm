use super::super::ast::{Argument, Expr, ExprKind, FStringSegment, Param, Value};
use super::super::interpreter::error::EldritchError;
use super::super::token::{Span, Token, TokenKind};
use super::Parser;
use alloc::boxed::Box;
use alloc::format;

use alloc::vec;
use alloc::vec::Vec;

impl Parser {
    // Helper to create Expr with span
    pub(crate) fn make_expr(&self, kind: ExprKind, start: Span, end: Span) -> Expr {
        let span = Span::new(start.start, end.end, start.line);
        Expr { kind, span }
    }

    pub(crate) fn expression(&mut self) -> Result<Expr, EldritchError> {
        // Handle ternary if expression: x if cond else y
        if self.match_token(&[TokenKind::Lambda]) {
            return self.lambda_expression();
        }

        let expr = self.logic_or()?;

        if self.match_token(&[TokenKind::If]) {
            let cond = self.logic_or()?;
            self.consume(|t| matches!(t, TokenKind::Else), "Expected 'else'.")?;
            let else_branch = self.expression()?;
            let start = expr.span;
            let end = else_branch.span;
            return Ok(self.make_expr(
                ExprKind::If {
                    cond: Box::new(cond),
                    then_branch: Box::new(expr),
                    else_branch: Box::new(else_branch),
                },
                start,
                end,
            ));
        }

        Ok(expr)
    }

    pub(crate) fn lambda_expression(&mut self) -> Result<Expr, EldritchError> {
        let start_span = self.tokens[self.current - 1].span;
        // Lambda params do not support type annotations in Python (and usually in dynamic languages)
        // because the colon is used to separate params from body.
        let params = self.parse_function_params(TokenKind::Colon, false)?;
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
    pub(crate) fn parse_function_params(
        &mut self,
        terminator: TokenKind,
        allow_annotations: bool,
    ) -> Result<Vec<Param>, EldritchError> {
        let mut params = Vec::new();
        if !self.check(&terminator) {
            loop {
                if self.match_token(&[TokenKind::Star]) {
                    let param_token = self.consume(
                        |t| matches!(t, TokenKind::Identifier(_)),
                        "Expected name after *.",
                    )?;
                    let param_name = if let TokenKind::Identifier(param_name) = &param_token.kind {
                        param_name.clone()
                    } else {
                        unreachable!()
                    };

                    // Handle type annotation for *args: *args: type
                    let annotation = if allow_annotations && self.match_token(&[TokenKind::Colon]) {
                        Some(Box::new(self.expression()?))
                    } else {
                        None
                    };

                    params.push(Param::Star(param_name, annotation));
                } else if self.match_token(&[TokenKind::StarStar]) {
                    let param_token = self.consume(
                        |t| matches!(t, TokenKind::Identifier(_)),
                        "Expected name after **.",
                    )?;
                    let param_name = if let TokenKind::Identifier(param_name) = &param_token.kind {
                        param_name.clone()
                    } else {
                        unreachable!()
                    };

                    // Handle type annotation for **kwargs: **kwargs: type
                    let annotation = if allow_annotations && self.match_token(&[TokenKind::Colon]) {
                        Some(Box::new(self.expression()?))
                    } else {
                        None
                    };

                    params.push(Param::StarStar(param_name, annotation));
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

                    // Parse optional type annotation
                    let annotation = if allow_annotations && self.match_token(&[TokenKind::Colon]) {
                        Some(Box::new(self.expression()?))
                    } else {
                        None
                    };

                    if self.match_token(&[TokenKind::Assign]) {
                        let default_val = self.expression()?;
                        params.push(Param::WithDefault(param_name, annotation, default_val));
                    } else {
                        // Check if a normal parameter follows a default parameter
                        let has_default =
                            params.iter().any(|p| matches!(p, Param::WithDefault(..)));
                        let has_star = params.iter().any(|p| {
                            matches!(p, Param::Star(..)) || matches!(p, Param::StarStar(..))
                        });

                        // Only an error if we haven't seen *args yet.
                        if has_default && !has_star {
                            return self.error("Non-default argument follows default argument.");
                        }

                        params.push(Param::Normal(param_name, annotation));
                    }
                }

                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }

                if self.check(&TokenKind::RParen) {
                    break;
                }
            }
        }
        Ok(params)
    }

    fn logic_or(&mut self) -> Result<Expr, EldritchError> {
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

    fn logic_and(&mut self) -> Result<Expr, EldritchError> {
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

    fn logic_not(&mut self) -> Result<Expr, EldritchError> {
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

    fn equality(&mut self) -> Result<Expr, EldritchError> {
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

    fn comparison(&mut self) -> Result<Expr, EldritchError> {
        let mut expr = self.bitwise_or()?;

        loop {
            let operator = if self.match_token(&[
                TokenKind::Lt,
                TokenKind::Gt,
                TokenKind::LtEq,
                TokenKind::GtEq,
                TokenKind::In,
            ]) {
                self.tokens[self.current - 1].clone()
            } else if self.check(&TokenKind::Not) {
                // Check if it's 'not in'. We need to peek ahead.
                // check() only checks current token. We need to check next token manually.
                // Or use peek_next()
                if matches!(self.peek_next().kind, TokenKind::In) {
                    self.advance(); // consume 'not'
                    self.advance(); // consume 'in'
                    // Create synthetic 'NotIn' token
                    let mut tok = self.tokens[self.current - 2].clone();
                    tok.kind = TokenKind::NotIn;
                    tok
                } else {
                    break;
                }
            } else {
                break;
            };

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

    fn bitwise_or(&mut self) -> Result<Expr, EldritchError> {
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

    fn bitwise_xor(&mut self) -> Result<Expr, EldritchError> {
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

    fn bitwise_and(&mut self) -> Result<Expr, EldritchError> {
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

    fn shift(&mut self) -> Result<Expr, EldritchError> {
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

    fn term(&mut self) -> Result<Expr, EldritchError> {
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

    fn factor(&mut self) -> Result<Expr, EldritchError> {
        let mut expr = self.unary_expr()?;
        while self.match_token(&[
            TokenKind::Slash,
            TokenKind::SlashSlash,
            TokenKind::Star,
            TokenKind::Percent,
        ]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.unary_expr()?;
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

    fn unary_expr(&mut self) -> Result<Expr, EldritchError> {
        if self.match_token(&[TokenKind::Minus, TokenKind::BitNot]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.unary_expr()?;
            let start = operator.span;
            let end = right.span;
            return Ok(self.make_expr(
                ExprKind::UnaryOp(operator.kind, Box::new(right)),
                start,
                end,
            ));
        }
        self.call()
    }

    fn call(&mut self) -> Result<Expr, EldritchError> {
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
                    if self.match_token(&[TokenKind::Colon]) && !self.check(&TokenKind::RBracket) {
                        step = Some(Box::new(self.expression()?));
                    }
                } else {
                    let first_expr = self.expression()?;

                    // Check for tuple indexing: a[1, 2]
                    if self.match_token(&[TokenKind::Comma]) {
                        let mut elements = vec![first_expr];
                        // Only continue if not immediately closed by RBracket (allow trailing comma if supported, or error if not expr)
                        loop {
                            if self.check(&TokenKind::RBracket) {
                                break;
                            }
                            elements.push(self.expression()?);
                            if !self.match_token(&[TokenKind::Comma]) {
                                break;
                            }
                        }
                        // Create a tuple expression for the index
                        let tuple_start = elements[0].span;
                        let tuple_end = elements.last().unwrap().span;
                        start = Some(Box::new(self.make_expr(
                            ExprKind::Tuple(elements),
                            tuple_start,
                            tuple_end,
                        )));
                    } else {
                        start = Some(Box::new(first_expr));
                        if self.match_token(&[TokenKind::Colon]) {
                            is_slice = true;
                            if !self.check(&TokenKind::Colon) && !self.check(&TokenKind::RBracket) {
                                stop = Some(Box::new(self.expression()?));
                            }
                            if self.match_token(&[TokenKind::Colon])
                                && !self.check(&TokenKind::RBracket)
                            {
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
                    if self.check(&TokenKind::Identifier(String::new())) {
                        let token = self.advance();
                        let n = if let TokenKind::Identifier(name) = &token.kind {
                            name.clone()
                        } else {
                            unreachable!()
                        };
                        (n, token.span)
                    } else {
                        // Handle missing identifier (e.g. "sys.") for autocomplete
                        // Use the dot token span (previous token) as the end span
                        let dot_span = self.tokens[self.current - 1].span;
                        (String::new(), dot_span)
                    }
                };
                expr = self.make_expr(ExprKind::GetAttr(Box::new(expr), name), start, end_span);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, EldritchError> {
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

                if self.check(&TokenKind::RParen) {
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

    fn primary(&mut self) -> Result<Expr, EldritchError> {
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

        if let TokenKind::Float(f) = token.kind {
            self.advance();
            return Ok(self.make_expr(ExprKind::Literal(Value::Float(f)), span, span));
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
                        if self.check(&TokenKind::RParen) {
                            break;
                        }
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

            // Attempt to parse first expression.
            // If it fails, we assume it's a List of expressions and try to recover.
            // But if it succeeds, we check for comprehension syntax.
            let first_expr_res = self.expression();

            // If first expression is valid, check for comprehension
            if let Ok(first_expr) = first_expr_res {
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
                    let iterable = self.logic_or()?;
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

                // Not a comprehension, so it's a list literal starting with first_expr
                let mut elements = vec![first_expr];
                if self.match_token(&[TokenKind::Comma]) && !self.check(&TokenKind::RBracket) {
                    loop {
                        if self.check(&TokenKind::RBracket) {
                            break;
                        }
                        match self.expression() {
                            Ok(e) => elements.push(e),
                            Err(e) => {
                                self.errors.push(e.clone());
                                // Recover: skip until comma or RBracket
                                while !self.check(&TokenKind::Comma)
                                    && !self.check(&TokenKind::RBracket)
                                    && !self.is_at_end()
                                {
                                    self.advance();
                                }
                                elements.push(self.make_expr(
                                    ExprKind::Error(e.message),
                                    e.span,
                                    e.span,
                                ));
                            }
                        }

                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
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
            } else {
                // First expression failed. Must be a list literal with error at start.
                let e = first_expr_res.unwrap_err();
                self.errors.push(e.clone());
                // Recover: skip until comma or RBracket
                while !self.check(&TokenKind::Comma)
                    && !self.check(&TokenKind::RBracket)
                    && !self.is_at_end()
                {
                    self.advance();
                }
                let err_expr = self.make_expr(ExprKind::Error(e.message), e.span, e.span);
                let mut elements = vec![err_expr];

                if self.match_token(&[TokenKind::Comma]) && !self.check(&TokenKind::RBracket) {
                    loop {
                        if self.check(&TokenKind::RBracket) {
                            break;
                        }
                        match self.expression() {
                            Ok(e) => elements.push(e),
                            Err(e) => {
                                self.errors.push(e.clone());
                                while !self.check(&TokenKind::Comma)
                                    && !self.check(&TokenKind::RBracket)
                                    && !self.is_at_end()
                                {
                                    self.advance();
                                }
                                elements.push(self.make_expr(
                                    ExprKind::Error(e.message),
                                    e.span,
                                    e.span,
                                ));
                            }
                        }
                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
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
        }

        if self.match_token(&[TokenKind::LBrace]) {
            if self.match_token(&[TokenKind::RBrace]) {
                let end = self.tokens[self.current - 1].span;
                // Empty braces {} usually mean empty dictionary in Python-like syntax
                return Ok(self.make_expr(ExprKind::Dictionary(Vec::new()), span, end));
            }

            let first_expr = self.expression()?;

            if self.match_token(&[TokenKind::Colon]) {
                // Dictionary or DictComp
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
                    // Use logic_or to avoid consuming the 'if' of the comprehension
                    let iterable = self.logic_or()?;
                    let mut cond = None;
                    if self.match_token(&[TokenKind::If]) {
                        cond = Some(Box::new(self.expression()?));
                    }
                    let end = self
                        .consume(|t| matches!(t, TokenKind::RBrace), "Expected '}'.")?
                        .span;
                    return Ok(self.make_expr(
                        ExprKind::DictComp {
                            key: Box::new(first_expr),
                            value: Box::new(val_expr),
                            var,
                            iterable: Box::new(iterable),
                            cond,
                        },
                        span,
                        end,
                    ));
                }

                let mut entries = vec![(first_expr, val_expr)];
                if self.match_token(&[TokenKind::Comma]) && !self.check(&TokenKind::RBrace) {
                    loop {
                        if self.check(&TokenKind::RBrace) {
                            break;
                        }
                        let k = self.expression()?;
                        self.consume(|t| matches!(t, TokenKind::Colon), "Expected ':'.")?;
                        let v = self.expression()?;
                        entries.push((k, v));
                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
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
            } else {
                // Set or SetComp
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
                    let iterable = self.logic_or()?;
                    let mut cond = None;
                    if self.match_token(&[TokenKind::If]) {
                        cond = Some(Box::new(self.expression()?));
                    }
                    let end = self
                        .consume(|t| matches!(t, TokenKind::RBrace), "Expected '}'.")?
                        .span;
                    return Ok(self.make_expr(
                        ExprKind::SetComp {
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
                if self.match_token(&[TokenKind::Comma]) && !self.check(&TokenKind::RBrace) {
                    loop {
                        if self.check(&TokenKind::RBrace) {
                            break;
                        }
                        elements.push(self.expression()?);
                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }
                let end = self
                    .consume(
                        |t| matches!(t, TokenKind::RBrace),
                        "Expected '}' after set definition.",
                    )?
                    .span;
                return Ok(self.make_expr(ExprKind::Set(elements), span, end));
            }
        }

        self.error(&format!("Unexpected token \"{}\"", self.peek().kind))
    }

    fn parse_fstring_content(
        &mut self,
        fstring_tokens: Vec<Token>,
        start_span: Span,
    ) -> Result<Expr, EldritchError> {
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
                return internal_parser.error(&format!(
                    "Unexpected token in f-string content: {:?}",
                    internal_parser.peek()
                ));
            }
        }
        // Use start_span as rough location
        Ok(self.make_expr(ExprKind::FString(segments), start_span, eof_span))
    }
}
