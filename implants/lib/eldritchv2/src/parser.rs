use super::ast::{Argument, Expr, FStringSegment, Param, Stmt, Value};
use super::token::Token;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec; // Added format! macro import

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

    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        core::mem::discriminant(self.peek()) == core::mem::discriminant(token)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    fn consume<F>(&mut self, check_fn: F, msg: &str) -> Result<&Token, String>
    where
        F: Fn(&Token) -> bool,
    {
        if check_fn(self.peek()) {
            Ok(self.advance())
        } else {
            Err(msg.to_string())
        }
    }

    fn match_token(&mut self, token_types: &[Token]) -> bool {
        for t in token_types {
            if core::mem::discriminant(self.peek()) == core::mem::discriminant(t) {
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
        matches!(&self.tokens[self.current], Token::Eof)
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            while matches!(self.peek(), Token::Newline) {
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
        if self.match_token(&[Token::Def]) {
            self.function_def()
        } else {
            self.statement()
        }
    }

    fn function_def(&mut self) -> Result<Stmt, String> {
        let name_token = self.consume(
            |t| matches!(t, Token::Identifier(_)),
            "Expected function name.",
        )?;
        let name = match name_token {
            Token::Identifier(s) => s.clone(),
            _ => unreachable!(),
        };

        self.consume(
            |t| matches!(t, Token::LParen),
            "Expected '(' after function name.",
        )?;

        let mut params = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                if self.match_token(&[Token::Star]) {
                    // *args
                    let param_token = self.consume(
                        |t| matches!(t, Token::Identifier(_)),
                        "Expected name after *.",
                    )?;
                    if let Token::Identifier(param_name) = param_token {
                        params.push(Param::Star(param_name.clone()));
                    }
                } else if self.match_token(&[Token::StarStar]) {
                    // **kwargs
                    let param_token = self.consume(
                        |t| matches!(t, Token::Identifier(_)),
                        "Expected name after **.",
                    )?;
                    if let Token::Identifier(param_name) = param_token {
                        params.push(Param::StarStar(param_name.clone()));
                    }
                } else {
                    let param_token = self.consume(
                        |t| matches!(t, Token::Identifier(_)),
                        "Expected parameter name.",
                    )?;
                    let param_name = if let Token::Identifier(name) = param_token {
                        name.clone()
                    } else {
                        unreachable!()
                    };

                    if self.match_token(&[Token::Assign]) {
                        let default_val = self.expression()?;
                        params.push(Param::WithDefault(param_name, default_val));
                    } else {
                        params.push(Param::Normal(param_name));
                    }
                }

                if !self.match_token(&[Token::Comma]) {
                    break;
                }
            }
        }

        self.consume(
            |t| matches!(t, Token::RParen),
            "Expected ')' after parameters.",
        )?;
        self.consume(
            |t| matches!(t, Token::Colon),
            "Expected ':' before function body.",
        )?;

        let body = self.parse_block_or_statement()?;
        Ok(Stmt::Def(name, params, body))
    }

    fn parse_block_or_statement(&mut self) -> Result<Vec<Stmt>, String> {
        if self.match_token(&[Token::Newline]) {
            self.consume(
                |t| matches!(t, Token::Indent),
                "Expected indentation after newline.",
            )?;
            let mut stmts = Vec::new();
            while !self.check(&Token::Dedent) && !self.is_at_end() {
                while matches!(self.peek(), Token::Newline) {
                    self.advance();
                }
                if self.check(&Token::Dedent) {
                    break;
                }
                stmts.push(self.declaration()?);
            }
            self.consume(
                |t| matches!(t, Token::Dedent),
                "Expected dedent after block.",
            )?;
            Ok(stmts)
        } else {
            let stmt = self.statement()?;
            Ok(vec![stmt])
        }
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(&[Token::If]) {
            self.if_statement()
        } else if self.match_token(&[Token::Return]) {
            self.return_statement()
        } else if self.match_token(&[Token::For]) {
            self.for_statement()
        } else if self.match_token(&[Token::Break]) {
            if !self.is_at_end() && !matches!(self.peek(), Token::Dedent) {
                self.consume(
                    |t| matches!(t, Token::Newline),
                    "Expected newline after break.",
                )?;
            }
            Ok(Stmt::Break)
        } else if self.match_token(&[Token::Continue]) {
            if !self.is_at_end() && !matches!(self.peek(), Token::Dedent) {
                self.consume(
                    |t| matches!(t, Token::Newline),
                    "Expected newline after continue.",
                )?;
            }
            Ok(Stmt::Continue)
        } else {
            self.expression_statement()
        }
    }

    fn for_statement(&mut self) -> Result<Stmt, String> {
        let ident_token = self.consume(
            |t| matches!(t, Token::Identifier(_)),
            "Expected iteration variable name after 'for'.",
        )?;
        let ident = match ident_token {
            Token::Identifier(s) => s.clone(),
            _ => unreachable!(),
        };
        self.consume(
            |t| matches!(t, Token::In),
            "Expected 'in' after iteration variable.",
        )?;
        let iterable = self.expression()?;
        self.consume(
            |t| matches!(t, Token::Colon),
            "Expected ':' before loop body.",
        )?;
        let body = self.parse_block_or_statement()?;
        Ok(Stmt::For(ident, iterable, body))
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        let condition = self.expression()?;
        self.consume(
            |t| matches!(t, Token::Colon),
            "Expected ':' after condition.",
        )?;
        let then_branch = self.parse_block_or_statement()?;
        let mut else_branch = None;

        if self.match_token(&[Token::Elif]) {
            let inner_if = self.if_statement()?;
            else_branch = Some(vec![inner_if]);
        } else if self.match_token(&[Token::Else]) {
            self.consume(|t| matches!(t, Token::Colon), "Expected ':' after else.")?;
            else_branch = Some(self.parse_block_or_statement()?);
        }
        Ok(Stmt::If(condition, then_branch, else_branch))
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        let mut value = None;
        if !self.check(&Token::Newline) && !self.check(&Token::Eof) && !self.check(&Token::Dedent) {
            value = Some(self.expression()?);
        }
        if !self.is_at_end() && !matches!(self.peek(), Token::Dedent) {
            self.consume(
                |t| matches!(t, Token::Newline),
                "Expected newline after return.",
            )?;
        }
        Ok(Stmt::Return(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        if self.match_token(&[Token::Assign]) {
            if let Expr::Identifier(name) = expr {
                let value = self.expression()?;
                if !self.is_at_end() && !matches!(self.peek(), Token::Dedent) {
                    self.consume(
                        |t| matches!(t, Token::Newline),
                        "Expected newline after assignment.",
                    )?;
                }
                return Ok(Stmt::Assignment(name, value));
            } else {
                return Err("Invalid assignment target.".to_string());
            }
        }
        if !self.is_at_end() && !matches!(self.peek(), Token::Dedent) {
            self.consume(
                |t| matches!(t, Token::Newline),
                "Expected newline after expression.",
            )?;
        }
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.logic_or()
    }

    fn logic_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.logic_and()?;
        while self.match_token(&[Token::Or]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.logic_and()?;
            expr = Expr::LogicalOp(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.logic_not()?;
        while self.match_token(&[Token::And]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.logic_not()?;
            expr = Expr::LogicalOp(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn logic_not(&mut self) -> Result<Expr, String> {
        if self.match_token(&[Token::Not]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.logic_not()?;
            return Ok(Expr::UnaryOp(operator, Box::new(right)));
        }
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;
        while self.match_token(&[Token::Eq, Token::NotEq]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.comparison()?;
            expr = Expr::BinaryOp(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.bitwise_or()?;
        while self.match_token(&[Token::Lt, Token::Gt, Token::LtEq, Token::GtEq]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.bitwise_or()?;
            expr = Expr::BinaryOp(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn bitwise_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.bitwise_xor()?;
        while self.match_token(&[Token::BitOr]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.bitwise_xor()?;
            expr = Expr::BinaryOp(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn bitwise_xor(&mut self) -> Result<Expr, String> {
        let mut expr = self.bitwise_and()?;
        while self.match_token(&[Token::BitXor]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.bitwise_and()?;
            expr = Expr::BinaryOp(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn bitwise_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.shift()?;
        while self.match_token(&[Token::BitAnd]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.shift()?;
            expr = Expr::BinaryOp(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn shift(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;
        while self.match_token(&[Token::LShift, Token::RShift]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.term()?;
            expr = Expr::BinaryOp(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;
        while self.match_token(&[Token::Minus, Token::Plus]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.factor()?;
            expr = Expr::BinaryOp(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        if self.match_token(&[Token::Minus, Token::BitNot]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.factor()?;
            return Ok(Expr::UnaryOp(operator, Box::new(right)));
        }

        let mut expr = self.call()?;
        while self.match_token(&[Token::Slash, Token::Star]) {
            let operator = self.tokens[self.current - 1].clone();
            let right = self.call()?;
            expr = Expr::BinaryOp(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[Token::LParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[Token::LBracket]) {
                let mut start = None;
                let mut stop = None;
                let mut step = None;
                let mut is_slice = false;

                if self.match_token(&[Token::Colon]) {
                    is_slice = true;
                    if !self.check(&Token::Colon) && !self.check(&Token::RBracket) {
                        stop = Some(Box::new(self.expression()?));
                    }
                    if self.match_token(&[Token::Colon]) {
                        if !self.check(&Token::RBracket) {
                            step = Some(Box::new(self.expression()?));
                        }
                    }
                } else {
                    start = Some(Box::new(self.expression()?));
                    if self.match_token(&[Token::Colon]) {
                        is_slice = true;
                        if !self.check(&Token::Colon) && !self.check(&Token::RBracket) {
                            stop = Some(Box::new(self.expression()?));
                        }
                        if self.match_token(&[Token::Colon]) {
                            if !self.check(&Token::RBracket) {
                                step = Some(Box::new(self.expression()?));
                            }
                        }
                    }
                }

                self.consume(
                    |t| matches!(t, Token::RBracket),
                    "Expected ']' after subscript.",
                )?;

                if is_slice {
                    expr = Expr::Slice(Box::new(expr), start, stop, step);
                } else {
                    expr = Expr::Index(Box::new(expr), start.unwrap());
                }
            } else if self.match_token(&[Token::Dot]) {
                let name_token = self.consume(
                    |t| matches!(t, Token::Identifier(_)),
                    "Expect property name after '.'.",
                )?;
                if let Token::Identifier(name) = name_token {
                    expr = Expr::GetAttr(Box::new(expr), name.clone());
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, String> {
        let mut args = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                let is_keyword = if let Token::Identifier(_) = self.peek() {
                    matches!(self.peek_next(), Token::Assign)
                } else {
                    false
                };

                if self.match_token(&[Token::Star]) {
                    let expr = self.expression()?;
                    args.push(Argument::StarArgs(expr));
                } else if self.match_token(&[Token::StarStar]) {
                    let expr = self.expression()?;
                    args.push(Argument::KwArgs(expr));
                } else if is_keyword {
                    let name = if let Token::Identifier(s) = self.advance() {
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

                if !self.match_token(&[Token::Comma]) {
                    break;
                }
            }
        }
        self.consume(
            |t| matches!(t, Token::RParen),
            "Expected ')' after arguments.",
        )?;
        Ok(Expr::Call(Box::new(callee), args))
    }

    fn primary(&mut self) -> Result<Expr, String> {
        if self.match_token(&[Token::False]) {
            return Ok(Expr::Literal(Value::Bool(false)));
        }
        if self.match_token(&[Token::True]) {
            return Ok(Expr::Literal(Value::Bool(true)));
        }
        if self.match_token(&[Token::None]) {
            return Ok(Expr::Literal(Value::None));
        }

        if self.match_token(&[Token::Integer(0)]) {
            if let Token::Integer(i) = self.tokens[self.current - 1] {
                return Ok(Expr::Literal(Value::Int(i)));
            }
        }

        if self.match_token(&[Token::String(String::new())]) {
            if let Token::String(s) = &self.tokens[self.current - 1] {
                return Ok(Expr::Literal(Value::String(s.clone())));
            }
        }

        if self.match_token(&[Token::FStringContent(Vec::new())]) {
            if let Token::FStringContent(fstring_tokens) = &self.tokens[self.current - 1] {
                return self.parse_fstring_content(fstring_tokens.clone());
            }
        }

        if self.match_token(&[Token::Identifier(String::new())]) {
            if let Token::Identifier(s) = &self.tokens[self.current - 1] {
                return Ok(Expr::Identifier(s.clone()));
            }
        }

        if self.match_token(&[Token::LParen]) {
            if self.match_token(&[Token::RParen]) {
                return Ok(Expr::Tuple(Vec::new()));
            }
            let expr = self.expression()?;
            if self.match_token(&[Token::Comma]) {
                let mut elements = vec![expr];
                if !self.check(&Token::RParen) {
                    loop {
                        elements.push(self.expression()?);
                        if !self.match_token(&[Token::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(|t| matches!(t, Token::RParen), "Expected ')' after tuple.")?;
                return Ok(Expr::Tuple(elements));
            }
            self.consume(
                |t| matches!(t, Token::RParen),
                "Expected ')' after expression.",
            )?;
            return Ok(expr);
        }

        if self.match_token(&[Token::LBracket]) {
            if self.match_token(&[Token::RBracket]) {
                return Ok(Expr::List(Vec::new()));
            }

            let first_expr = self.expression()?;

            if self.match_token(&[Token::For]) {
                let var_token = self.consume(
                    |t| matches!(t, Token::Identifier(_)),
                    "Expected iteration variable.",
                )?;
                let var = match var_token {
                    Token::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };
                self.consume(|t| matches!(t, Token::In), "Expected 'in'.")?;
                let iterable = self.expression()?;
                let mut cond = None;
                if self.match_token(&[Token::If]) {
                    cond = Some(Box::new(self.expression()?));
                }
                self.consume(|t| matches!(t, Token::RBracket), "Expected ']'.")?;
                return Ok(Expr::ListComp {
                    body: Box::new(first_expr),
                    var,
                    iterable: Box::new(iterable),
                    cond,
                });
            }

            let mut elements = vec![first_expr];
            if self.match_token(&[Token::Comma]) {
                if !self.check(&Token::RBracket) {
                    loop {
                        elements.push(self.expression()?);
                        if !self.match_token(&[Token::Comma]) {
                            break;
                        }
                    }
                }
            }
            self.consume(|t| matches!(t, Token::RBracket), "Expected ']' after list.")?;
            return Ok(Expr::List(elements));
        }

        if self.match_token(&[Token::LBrace]) {
            if self.match_token(&[Token::RBrace]) {
                return Ok(Expr::Dictionary(Vec::new()));
            }

            let key_expr = self.expression()?;
            self.consume(|t| matches!(t, Token::Colon), "Expected ':' after key.")?;
            let val_expr = self.expression()?;

            if self.match_token(&[Token::For]) {
                let var_token = self.consume(
                    |t| matches!(t, Token::Identifier(_)),
                    "Expected iteration variable.",
                )?;
                let var = match var_token {
                    Token::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };
                self.consume(|t| matches!(t, Token::In), "Expected 'in'.")?;
                let iterable = self.expression()?;
                let mut cond = None;
                if self.match_token(&[Token::If]) {
                    cond = Some(Box::new(self.expression()?));
                }
                self.consume(|t| matches!(t, Token::RBrace), "Expected '}'.")?;
                return Ok(Expr::DictComp {
                    key: Box::new(key_expr),
                    value: Box::new(val_expr),
                    var,
                    iterable: Box::new(iterable),
                    cond,
                });
            }

            let mut entries = vec![(key_expr, val_expr)];
            if self.match_token(&[Token::Comma]) {
                if !self.check(&Token::RBrace) {
                    loop {
                        let k = self.expression()?;
                        self.consume(|t| matches!(t, Token::Colon), "Expected ':'.")?;
                        let v = self.expression()?;
                        entries.push((k, v));
                        if !self.match_token(&[Token::Comma]) {
                            break;
                        }
                    }
                }
            }
            self.consume(
                |t| matches!(t, Token::RBrace),
                "Expected '}' after dictionary definition.",
            )?;
            return Ok(Expr::Dictionary(entries));
        }

        Err(format!("Expect expression. Found {:?}", self.peek()))
    }

    fn parse_fstring_content(&mut self, fstring_tokens: Vec<Token>) -> Result<Expr, String> {
        let mut tokens_with_eof = fstring_tokens;
        tokens_with_eof.push(Token::Eof);

        let mut internal_parser = Parser::new(tokens_with_eof);
        let mut segments = Vec::new();

        while !internal_parser.is_at_end() {
            if let Token::String(s) = internal_parser.peek() {
                segments.push(FStringSegment::Literal(s.clone()));
                internal_parser.advance();
            } else if internal_parser.match_token(&[Token::LParen]) {
                let expr = internal_parser.expression()?;
                segments.push(FStringSegment::Expression(expr));
                internal_parser.consume(
                    |t| matches!(t, Token::RParen),
                    "Expected ')' to close f-string embedded expression.",
                )?;
            } else {
                return Err(format!(
                    "Unexpected token in f-string content: {:?}",
                    internal_parser.peek()
                ));
            }
        }
        Ok(Expr::FString(segments))
    }
}
