use crate::ast::{BlockStatement, Expression, Program, Statement};
use crate::lexer::{Lexer, Token};
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

#[derive(Debug)]
pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
    errors: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Lowest,
    Equals,      // == or !=
    LessGreater, // < or >
    Sum,         // + or -
    Product,     // * or /
    Prefix,      // -X or !X
    Call,        // fn(x)
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        // Initialize current_token and peek_token
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();

        Parser {
            lexer,
            current_token,
            peek_token,
            errors: Vec::new(),
        }
    }

    pub fn errors(&self) -> &Vec<String> {
        &self.errors
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    pub fn parse_program(&mut self) -> Program {
        let mut statements = Vec::new();

        while self.current_token != Token::Eof {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        Program { statements }
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.current_token {
            Token::Identifier(_) if self.peek_token == Token::Assign => {
                self.parse_assignment_statement()
            }
            Token::Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_assignment_statement(&mut self) -> Option<Statement> {
        // current_token should be an identifier: the variable name.
        let name = match self.current_token.clone() {
            Token::Identifier(name) => name,
            other => {
                self.errors
                    .push(format!("expected identifier before '=', got {:?}", other));
                return None;
            }
        };

        // Expect '=' after the identifier.
        if !self.expect_peek(&Token::Assign) {
            return None;
        }

        // Move to the first token of the right-hand side expression.
        self.next_token();

        let value = match self.parse_expression(Precedence::Lowest) {
            Some(expr) => expr,
            None => return None,
        };

        // Optional trailing semicolon: `x = 10;` is allowed.
        if self.peek_token == Token::Semicolon {
            self.next_token();
        }

        Some(Statement::Assign(name, value))
    }

    fn parse_return_statement(&mut self) -> Option<Statement> {
        // current_token is 'return'
        self.next_token(); // move to first token of the expression

        // For now, require an expression after `return`
        if let Some(expr) = self.parse_expression(Precedence::Lowest) {
            if self.peek_token == Token::Semicolon {
                self.next_token();
            }
            Some(Statement::Return(expr))
        } else {
            self.errors
                .push("expected expression after 'return'".to_string());
            None
        }
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let expr = match self.parse_expression(Precedence::Lowest) {
            Some(e) => e,
            None => return None,
        };

        if self.peek_token == Token::Semicolon {
            self.next_token();
        }

        Some(Statement::Expression(expr))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        // Parse prefix
        let mut left = match self.current_token.clone() {
            Token::Identifier(name) => Some(Expression::Identifier(name)),
            Token::Integer(value) => Some(Expression::IntegerLiteral(value)),
            Token::String(s) => Some(Expression::StringLiteral(s)),
            Token::True => Some(Expression::Boolean(true)),
            Token::False => Some(Expression::Boolean(false)),
            Token::Bang | Token::Minus => self.parse_prefix_expression(),
            Token::LeftParen => self.parse_grouped_expression(),
            Token::If => self.parse_if_expression(),
            Token::Function => self.parse_function_literal(),
            other => {
                self.no_prefix_parse_fn_error(other);
                None
            }
        }?;

        // Parse infix / postfix (Pratt loop)
        while self.peek_token != Token::Semicolon && precedence < self.peek_precedence() {
            match self.peek_token.clone() {
                Token::Plus
                | Token::Minus
                | Token::Slash
                | Token::Asterisk
                | Token::Equal
                | Token::NotEqual
                | Token::LessThan
                | Token::GreaterThan => {
                    self.next_token();
                    left = self.parse_infix_expression(left)?;
                }
                Token::LeftParen => {
                    self.next_token();
                    left = self.parse_call_expression(left)?;
                }
                _ => {
                    return Some(left);
                }
            }
        }

        Some(left)
    }

    fn parse_prefix_expression(&mut self) -> Option<Expression> {
        let operator = self.current_token.clone();

        self.next_token();

        let right = self.parse_expression(Precedence::Prefix)?;

        Some(Expression::Prefix(operator, Box::new(right)))
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Option<Expression> {
        let operator = self.current_token.clone();
        let precedence = self.current_precedence();

        self.next_token();

        let right = self.parse_expression(precedence)?;

        Some(Expression::Infix(operator, Box::new(left), Box::new(right)))
    }

    fn parse_grouped_expression(&mut self) -> Option<Expression> {
        self.next_token(); // skip '('

        let expr = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(&Token::RightParen) {
            return None;
        }

        Some(expr)
    }

    fn parse_if_expression(&mut self) -> Option<Expression> {
        // current_token is 'if'
        if !self.expect_peek(&Token::LeftParen) {
            return None;
        }

        self.next_token(); // move to condition
        let condition = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(&Token::RightParen) {
            return None;
        }

        if !self.expect_peek(&Token::LeftBrace) {
            return None;
        }

        let consequence = self.parse_block_statement();

        let alternative = if self.peek_token == Token::Else {
            self.next_token(); // move to 'else'
            if !self.expect_peek(&Token::LeftBrace) {
                return None;
            }
            Some(self.parse_block_statement())
        } else {
            None
        };

        Some(Expression::If {
            condition: Box::new(condition),
            consequence,
            alternative,
        })
    }

    fn parse_block_statement(&mut self) -> BlockStatement {
        // current_token is '{'
        self.next_token(); // move to first token inside block

        let mut statements = Vec::new();

        while self.current_token != Token::RightBrace && self.current_token != Token::Eof {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        BlockStatement { statements }
    }

    fn parse_function_literal(&mut self) -> Option<Expression> {
        // current_token is 'fn'
        if !self.expect_peek(&Token::LeftParen) {
            return None;
        }

        let parameters = self.parse_function_parameters()?;

        if !self.expect_peek(&Token::LeftBrace) {
            return None;
        }

        let body = self.parse_block_statement();

        Some(Expression::FunctionLiteral { parameters, body })
    }

    fn parse_function_parameters(&mut self) -> Option<Vec<String>> {
        let mut identifiers = Vec::new();

        // No parameters: fn()
        if self.peek_token == Token::RightParen {
            self.next_token(); // consume ')'
            return Some(identifiers);
        }

        self.next_token(); // move to first identifier

        match &self.current_token {
            Token::Identifier(name) => identifiers.push(name.clone()),
            _ => {
                self.peek_error("identifier".to_string());
                return None;
            }
        }

        while self.peek_token == Token::Comma {
            self.next_token(); // current is ','
            self.next_token(); // move to next identifier

            match &self.current_token {
                Token::Identifier(name) => identifiers.push(name.clone()),
                _ => {
                    self.peek_error("identifier".to_string());
                    return None;
                }
            }
        }

        if !self.expect_peek(&Token::RightParen) {
            return None;
        }

        Some(identifiers)
    }

    fn parse_call_expression(&mut self, function: Expression) -> Option<Expression> {
        let arguments = self.parse_call_arguments()?;
        Some(Expression::Call {
            function: Box::new(function),
            arguments,
        })
    }

    fn parse_call_arguments(&mut self) -> Option<Vec<Expression>> {
        let mut args = Vec::new();

        if self.peek_token == Token::RightParen {
            self.next_token(); // consume ')'
            return Some(args);
        }

        self.next_token(); // first argument
        args.push(self.parse_expression(Precedence::Lowest)?);

        while self.peek_token == Token::Comma {
            self.next_token(); // ','
            self.next_token(); // next argument
            args.push(self.parse_expression(Precedence::Lowest)?);
        }

        if !self.expect_peek(&Token::RightParen) {
            return None;
        }

        Some(args)
    }

    fn expect_peek(&mut self, expected: &Token) -> bool {
        if self.peek_token == *expected {
            self.next_token();
            true
        } else {
            self.errors.push(format!(
                "expected next token to be {:?}, got {:?} instead",
                expected, self.peek_token
            ));
            false
        }
    }

    fn peek_error(&mut self, expected: String) {
        self.errors.push(format!(
            "expected next token to be {}, got {:?} instead",
            expected, self.peek_token
        ));
    }

    fn no_prefix_parse_fn_error(&mut self, token: Token) {
        self.errors.push(format!(
            "no prefix parse function for token {:?} found",
            token
        ));
    }

    fn peek_precedence(&self) -> Precedence {
        precedence_of(&self.peek_token)
    }

    fn current_precedence(&self) -> Precedence {
        precedence_of(&self.current_token)
    }
}

fn precedence_of(token: &Token) -> Precedence {
    match token {
        Token::Equal | Token::NotEqual => Precedence::Equals,
        Token::LessThan | Token::GreaterThan => Precedence::LessGreater,
        Token::Plus | Token::Minus => Precedence::Sum,
        Token::Slash | Token::Asterisk => Precedence::Product,
        Token::LeftParen => Precedence::Call,
        _ => Precedence::Lowest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expression, Statement};
    use crate::lexer::{Lexer, Token};

    pub fn check_parser_errors(parser: &Parser) {
        if parser.errors().is_empty() {
            return;
        }
        for err in parser.errors() {
            eprintln!("parser error: {}", err);
        }
        panic!("parser had {} error(s)", parser.errors().len());
    }

    #[test]
    fn test_parse_let_statements() {
        let input = r#"
            let x = 5;
            let y = 10;
            let foobar = x + y;
        "#;

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        check_parser_errors(&parser);

        assert_eq!(
            program.statements.len(),
            3,
            "program.statements has wrong length: got {}, want 3",
            program.statements.len()
        );

        let expected_identifiers = ["x", "y", "foobar"];

        for (stmt, expected_name) in program.statements.iter().zip(expected_identifiers.iter()) {
            match stmt {
                Statement::Let(name, value) => {
                    assert_eq!(
                        name, expected_name,
                        "let statement has wrong name. got={}, want={}",
                        name, expected_name
                    );
                    // smoke check: value is at least *some* expression
                    match value {
                        Expression::IntegerLiteral(_) => {}
                        Expression::Infix(_, _, _) => {}
                        _ => {} // accept other expressions too
                    }
                }
                other => panic!("expected let statement, got: {:?}", other),
            }
        }
    }

    #[test]
    fn test_operator_precedence_parsing() {
        // This checks that the Pratt/precedence logic is wired correctly:
        // 1 + 2 * 3 == 7 should parse as:
        // ((1) + (2 * 3)) == 7
        let input = "1 + 2 * 3 == 7;";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        check_parser_errors(&parser);

        assert_eq!(
            program.statements.len(),
            1,
            "expected 1 statement, got {}",
            program.statements.len()
        );

        // Unwrap the single expression statement
        let expr = match &program.statements[0] {
            Statement::Expression(expr) => expr,
            other => panic!("expected expression statement, got {:?}", other),
        };

        // Top-level should be == infix expression
        match expr {
            Expression::Infix(Token::Equal, left, right) => {
                // right side: 7
                match &**right {
                    Expression::IntegerLiteral(v) => assert_eq!(*v, 7),
                    other => panic!("expected integer literal on right, got {:?}", other),
                }

                // left side: 1 + 2 * 3
                match &**left {
                    Expression::Infix(Token::Plus, left_inner, right_inner) => {
                        // left_inner: 1
                        match &**left_inner {
                            Expression::IntegerLiteral(v) => assert_eq!(*v, 1),
                            other => panic!("expected 1 on left_inner, got {:?}", other),
                        }

                        // right_inner: 2 * 3
                        match &**right_inner {
                            Expression::Infix(Token::Asterisk, l, r) => {
                                match &**l {
                                    Expression::IntegerLiteral(v) => assert_eq!(*v, 2),
                                    other => {
                                        panic!("expected 2 on left of *, got {:?}", other)
                                    }
                                }
                                match &**r {
                                    Expression::IntegerLiteral(v) => assert_eq!(*v, 3),
                                    other => {
                                        panic!("expected 3 on right of *, got {:?}", other)
                                    }
                                }
                            }
                            other => panic!("expected * infix, got {:?}", other),
                        }
                    }
                    other => panic!("expected + infix, got {:?}", other),
                }
            }
            other => panic!("expected == infix, got {:?}", other),
        }
    }
}
