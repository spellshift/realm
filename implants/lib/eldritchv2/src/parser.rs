use super::ast::{Expr, FStringSegment, Stmt, Value};
use super::token::Token;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    // --- Helpers ---

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn check(&self, token: &Token) -> bool {
        // Must check bounds first to prevent panic in peek()
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(self.peek()) == std::mem::discriminant(token)
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
            if std::mem::discriminant(self.peek()) == std::mem::discriminant(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn is_at_end(&self) -> bool {
        // FIX: Check bounds first to prevent index out of bounds panic in self.peek().
        if self.current >= self.tokens.len() {
            return true;
        }

        // If within bounds, check if the token is the Eof marker.
        matches!(&self.tokens[self.current], Token::Eof)
    }

    // --- Grammar Rules ---

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

        let mut parameters = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                let param_token = self.consume(
                    |t| matches!(t, Token::Identifier(_)),
                    "Expected parameter name.",
                )?;
                if let Token::Identifier(param_name) = param_token {
                    parameters.push(param_name.clone());
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
        self.consume(
            |t| matches!(t, Token::Newline),
            "Expected newline before block.",
        )?;
        self.consume(
            |t| matches!(t, Token::Indent),
            "Expected indentation for function body.",
        )?;

        let body = self.block()?;

        Ok(Stmt::Def(name, parameters, body))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, String> {
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
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(&[Token::If]) {
            self.if_statement()
        } else if self.match_token(&[Token::Return]) {
            self.return_statement()
        } else if self.match_token(&[Token::For]) {
            self.for_statement()
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
        self.consume(
            |t| matches!(t, Token::Newline),
            "Expected newline before loop block.",
        )?;
        self.consume(
            |t| matches!(t, Token::Indent),
            "Expected indentation for loop body.",
        )?;

        let body = self.block()?;

        Ok(Stmt::For(ident, iterable, body))
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        let condition = self.expression()?;
        self.consume(
            |t| matches!(t, Token::Colon),
            "Expected ':' after condition.",
        )?;
        self.consume(|t| matches!(t, Token::Newline), "Expected newline.")?;
        self.consume(|t| matches!(t, Token::Indent), "Expected indent.")?;

        let then_branch = self.block()?;
        let mut else_branch = None;

        if self.match_token(&[Token::Elif]) {
            // Elif is an 'else' that contains a nested 'if' statement (the 'else' body is just the inner 'if')
            let inner_if = self.if_statement()?;
            else_branch = Some(vec![inner_if]);
        } else if self.match_token(&[Token::Else]) {
            self.consume(|t| matches!(t, Token::Colon), "Expected ':' after else.")?;
            self.consume(|t| matches!(t, Token::Newline), "Expected newline.")?;
            self.consume(|t| matches!(t, Token::Indent), "Expected indent.")?;
            else_branch = Some(self.block()?);
        }

        Ok(Stmt::If(condition, then_branch, else_branch))
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        let mut value = None;
        if !self.check(&Token::Newline) {
            value = Some(self.expression()?);
        }
        self.consume(
            |t| matches!(t, Token::Newline) || matches!(t, Token::Eof),
            "Expected newline after return.",
        )?;
        Ok(Stmt::Return(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;

        if self.match_token(&[Token::Assign]) {
            if let Expr::Identifier(name) = expr {
                let value = self.expression()?;
                self.consume(
                    |t| matches!(t, Token::Newline) || matches!(t, Token::Eof),
                    "Expected newline after assignment.",
                )?;
                return Ok(Stmt::Assignment(name, value));
            } else {
                return Err("Invalid assignment target.".to_string());
            }
        }

        self.consume(
            |t| matches!(t, Token::Newline) || matches!(t, Token::Eof),
            "Expected newline after expression.",
        )?;
        Ok(Stmt::Expression(expr))
    }

    // --- Expression Parsing (Precedence) ---

    fn expression(&mut self) -> Result<Expr, String> {
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
        let mut expr = self.term()?;
        while self.match_token(&[Token::Lt, Token::Gt]) {
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
                // New: Handle Indexing / Subscripting [index]
                let index = self.expression()?;
                self.consume(
                    |t| matches!(t, Token::RBracket),
                    "Expected ']' after subscript.",
                )?;
                expr = Expr::Index(Box::new(expr), Box::new(index));
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
                args.push(self.expression()?);
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

        // Handle normal strings
        if self.match_token(&[Token::String(String::new())]) {
            if let Token::String(s) = &self.tokens[self.current - 1] {
                return Ok(Expr::Literal(Value::String(s.clone())));
            }
        }

        // Handle FStrings using the token stream provided by the lexer
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
            let expr = self.expression()?;
            self.consume(
                |t| matches!(t, Token::RParen),
                "Expected ')' after expression.",
            )?;
            return Ok(expr);
        }

        // --- List Literal ---
        if self.match_token(&[Token::LBracket]) {
            let mut elements = Vec::new();
            if !self.check(&Token::RBracket) {
                loop {
                    elements.push(self.expression()?);
                    if !self.match_token(&[Token::Comma]) {
                        break;
                    }
                }
            }
            self.consume(|t| matches!(t, Token::RBracket), "Expected ']' after list.")?;
            return Ok(Expr::List(elements));
        }

        // --- Dictionary Literal ---
        if self.match_token(&[Token::LBrace]) {
            let mut entries = Vec::new();
            if !self.check(&Token::RBrace) {
                loop {
                    // Key must be an expression (usually a string literal or identifier)
                    let key = self.expression()?;

                    self.consume(
                        |t| matches!(t, Token::Colon),
                        "Expected ':' after dictionary key.",
                    )?;

                    // Value is an expression
                    let value = self.expression()?;
                    entries.push((key, value));

                    if !self.match_token(&[Token::Comma]) {
                        break;
                    }
                }
            }
            self.consume(
                |t| matches!(t, Token::RBrace),
                "Expected '}' after dictionary definition.",
            )?;
            return Ok(Expr::Dictionary(entries));
        }
        // --- End Dictionary Literal ---

        Err(format!("Expect expression. Found {:?}", self.peek()))
    }

    // Parses the token stream generated inside an f-string by the lexer
    fn parse_fstring_content(&mut self, fstring_tokens: Vec<Token>) -> Result<Expr, String> {
        // The internal token stream lacks an Eof token, which causes the internal parser
        // to panic when it reaches the end. We append one manually for safety.
        let mut tokens_with_eof = fstring_tokens;
        tokens_with_eof.push(Token::Eof);

        let mut internal_parser = Parser::new(tokens_with_eof);
        let mut segments = Vec::new();

        while !internal_parser.is_at_end() {
            // Check for the literal string part first (must be a String token from the lexer)
            if let Token::String(s) = internal_parser.peek() {
                segments.push(FStringSegment::Literal(s.clone()));
                internal_parser.advance();
            } else if internal_parser.match_token(&[Token::LParen]) {
                // If it's a LParen, it signals the start of an embedded expression
                // (The lexer replaces '{' with LParen and '}' with RParen around the expression tokens)

                // Parse the expression using the temporary parser's rules
                let expr = internal_parser.expression()?;
                segments.push(FStringSegment::Expression(expr));

                // Consume the closing RParen (which was the '}')
                internal_parser.consume(
                    |t| matches!(t, Token::RParen),
                    "Expected ')' to close f-string embedded expression.",
                )?;
            } else {
                // This path should ideally never be reached if the lexer is correct and we handle Eof
                return Err(format!(
                    "Unexpected token in f-string content: {:?}",
                    internal_parser.peek()
                ));
            }
        }

        Ok(Expr::FString(segments))
    }
}
