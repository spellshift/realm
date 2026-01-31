use super::token::{Span, Token, TokenKind};
use alloc::collections::VecDeque;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

pub struct Lexer {
    source: Vec<char>,
    start: usize,
    current: usize,
    line: usize,
    indent_stack: Vec<usize>,
    nesting: usize,
    pending_tokens: VecDeque<Token>,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        let mut chars: Vec<char> = source.chars().collect();
        chars.push('\n');

        Lexer {
            source: chars,
            start: 0,
            current: 0,
            line: 1,
            indent_stack: vec![0],
            nesting: 0,
            pending_tokens: VecDeque::new(),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let character = self.source[self.current];
        self.current += 1;
        character
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.current]
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source[self.current + 1]
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn add_token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            span: Span::new(self.start, self.current, self.line),
        }
    }

    fn error_token(&self, message: &str) -> Token {
        Token {
            kind: TokenKind::Error(String::from(message)),
            span: Span::new(self.start, self.current, self.line),
        }
    }

    fn skip_comment(&mut self) {
        while self.peek() != '\n' && !self.is_at_end() {
            self.advance();
        }
    }

    fn identifier(&mut self) -> Token {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        let value: String = self.source[self.start..self.current].iter().collect();
        if let Some(kind) = TokenKind::from_keyword(&value) {
            self.add_token(kind)
        } else {
            self.add_token(TokenKind::Identifier(value))
        }
    }

    fn number(&mut self) -> Token {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume the "."
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }

            let value: String = self.source[self.start..self.current].iter().collect();
            let float_val: f64 = value.parse().unwrap_or(0.0);
            return self.add_token(TokenKind::Float(float_val));
        }

        let value: String = self.source[self.start..self.current].iter().collect();
        let number: i64 = value.parse().unwrap_or(0);
        self.add_token(TokenKind::Integer(number))
    }

    fn string(
        &mut self,
        quote_char: char,
        is_fstring: bool,
        is_bytes: bool,
        is_raw: bool,
    ) -> Token {
        if is_fstring || is_bytes || is_raw {
            self.start = self.current;
        } else {
            self.start += 1;
        }

        let mut fstring_tokens = Vec::new();
        let mut current_literal = String::new();

        let is_triple = if self.peek() == quote_char && self.peek_next() == quote_char {
            self.advance();
            self.advance();
            true
        } else {
            false
        };

        loop {
            if self.is_at_end() {
                return self.error_token(&format!(
                    "Unterminated string literal on line {}",
                    self.line
                ));
            }

            let c = self.peek();

            if c == quote_char {
                if is_triple {
                    if self.peek_next() == quote_char
                        && self.current + 2 < self.source.len()
                        && self.source[self.current + 2] == quote_char
                    {
                        self.advance();
                        self.advance();
                        self.advance();
                        break;
                    }
                } else {
                    self.advance();
                    break;
                }
            }

            if c == '\n' {
                if !is_triple {
                    return self.error_token(&format!(
                        "Unterminated string literal (newline) on line {}",
                        self.line
                    ));
                }
                self.line += 1;
            }

            if c == '{' && is_fstring && !is_bytes && !is_raw {
                if !current_literal.is_empty() {
                    fstring_tokens.push(self.create_string_token(current_literal.clone()));
                    current_literal.clear();
                }
                self.advance();
                let expr_tokens = self.tokenize_fstring_expression();
                fstring_tokens.extend(expr_tokens);
                continue;
            }

            if c == '\\' {
                self.advance();
                if self.is_at_end() {
                    return self.error_token("Unterminated string literal");
                }

                if is_raw {
                    let next_char = self.peek();
                    if next_char == quote_char {
                        // Escape the quote, but keep the backslash in the string
                        current_literal.push('\\');
                        current_literal.push(self.advance());
                    } else {
                        // Keep the backslash, don't consume next char yet
                        current_literal.push('\\');
                        if next_char == '\\' {
                            // Consume the second backslash so it doesn't escape anything else
                            current_literal.push(self.advance());
                        }
                    }
                } else {
                    let escaped = self.advance();
                    match escaped {
                        'n' => current_literal.push('\n'),
                        't' => current_literal.push('\t'),
                        'r' => current_literal.push('\r'),
                        '\\' => current_literal.push('\\'),
                        '"' => current_literal.push('"'),
                        '\'' => current_literal.push('\''),
                        '\n' => {
                            self.line += 1;
                        }
                        c => current_literal.push(c),
                    }
                }
            } else {
                current_literal.push(self.advance());
            }
        }

        if is_bytes {
            let bytes: Vec<u8> = current_literal.chars().map(|c| c as u8).collect();
            self.add_token(TokenKind::Bytes(bytes))
        } else if is_fstring {
            if !current_literal.is_empty() {
                fstring_tokens.push(self.create_string_token(current_literal));
            }
            self.add_token(TokenKind::FStringContent(fstring_tokens))
        } else {
            self.add_token(TokenKind::String(current_literal))
        }
    }

    fn create_string_token(&self, s: String) -> Token {
        Token {
            kind: TokenKind::String(s),
            span: Span::new(self.current, self.current, self.line),
        }
    }

    fn tokenize_fstring_expression(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let initial_start = self.current;
        let mut nesting_level = 1;

        while nesting_level > 0 && !self.is_at_end() && self.peek() != '\n' {
            if self.peek() == '{' {
                nesting_level += 1;
            } else if self.peek() == '}' {
                nesting_level -= 1;
            }
            if nesting_level > 0 {
                self.advance();
            }
        }

        if nesting_level > 0 {
            return vec![self.error_token(&format!(
                "Unmatched '{{' in f-string expression starting at line {}",
                self.line
            ))];
        }

        let end_of_expr = self.current;
        let expr_source: String = self.source[initial_start..end_of_expr].iter().collect();
        let mut expr_lexer = Lexer::new(expr_source);

        // Recursively tokenize the expression inside the f-string
        let sub_tokens = expr_lexer.scan_tokens();
        for token in sub_tokens {
            match token.kind {
                TokenKind::Eof => break,
                TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent => continue,
                TokenKind::String(s) => tokens.push(Token {
                    kind: TokenKind::String(s),
                    span: token.span,
                }),
                _ => tokens.push(token),
            }
        }

        self.advance();

        let mut final_tokens = vec![self.add_token(TokenKind::LParen)];
        final_tokens.extend(tokens);
        final_tokens.push(self.add_token(TokenKind::RParen));
        final_tokens
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            match token.kind {
                TokenKind::Eof => {
                    while *self.indent_stack.last().unwrap() > 0 {
                        self.indent_stack.pop();
                        tokens.push(self.add_token(TokenKind::Dedent));
                    }
                    tokens.push(self.add_token(TokenKind::Eof));
                    return tokens;
                }
                _ => tokens.push(token),
            }
        }
    }

    fn next_token(&mut self) -> Token {
        if let Some(token) = self.pending_tokens.pop_front() {
            return token;
        }

        self.start = self.current;

        while self.peek() == '\n' && self.nesting > 0 {
            self.advance();
            self.line += 1;
        }

        if self.start > 0 && self.source[self.start - 1] == '\n' {
            let mut indent_count = 0;
            while self.peek().is_whitespace() && self.peek() != '\n' {
                self.advance();
                indent_count += 1;
            }
            if self.is_at_end() {
                return self.add_token(TokenKind::Eof);
            }

            if self.peek() != '\n' {
                let current_indent = *self.indent_stack.last().unwrap();
                if indent_count > current_indent {
                    self.indent_stack.push(indent_count);
                    return self.add_token(TokenKind::Indent);
                } else if indent_count < current_indent {
                    let mut dedents = Vec::new();
                    while *self.indent_stack.last().unwrap() > indent_count {
                        self.indent_stack.pop();
                        dedents.push(self.add_token(TokenKind::Dedent));
                    }
                    if *self.indent_stack.last().unwrap() != indent_count {
                        return self.error_token(&format!(
                            "Inconsistent indentation on line {}",
                            self.line
                        ));
                    }
                    self.current = self.start + indent_count;
                    if !dedents.is_empty() {
                        let first = dedents.remove(0);
                        for t in dedents {
                            self.pending_tokens.push_back(t);
                        }
                        return first;
                    }
                }
            } else {
                self.advance();
                self.line += 1;
                return self.next_token();
            }
        }

        loop {
            while self.peek().is_whitespace() && self.peek() != '\n' {
                self.advance();
            }

            if self.peek() == '\n' && self.nesting > 0 {
                self.advance();
                self.line += 1;
                continue;
            }

            break;
        }
        self.start = self.current;
        if self.is_at_end() {
            return self.add_token(TokenKind::Eof);
        }

        let c = self.advance();

        match c {
            '(' => {
                self.nesting += 1;
                self.add_token(TokenKind::LParen)
            }
            ')' => {
                if self.nesting > 0 {
                    self.nesting -= 1;
                }
                self.add_token(TokenKind::RParen)
            }
            '[' => {
                self.nesting += 1;
                self.add_token(TokenKind::LBracket)
            }
            ']' => {
                if self.nesting > 0 {
                    self.nesting -= 1;
                }
                self.add_token(TokenKind::RBracket)
            }
            '{' => {
                self.nesting += 1;
                self.add_token(TokenKind::LBrace)
            }
            '}' => {
                if self.nesting > 0 {
                    self.nesting -= 1;
                }
                self.add_token(TokenKind::RBrace)
            }
            ',' => self.add_token(TokenKind::Comma),
            ':' => self.add_token(TokenKind::Colon),
            '.' => {
                // Check for leading dot float: .5
                if self.peek().is_ascii_digit() {
                    self.start = self.current - 1; // Include the dot
                    while self.peek().is_ascii_digit() {
                        self.advance();
                    }
                    let value: String = self.source[self.start..self.current].iter().collect();
                    let float_val: f64 = value.parse().unwrap_or(0.0);
                    self.add_token(TokenKind::Float(float_val))
                } else {
                    self.add_token(TokenKind::Dot)
                }
            }
            ';' => self.add_token(TokenKind::Newline),
            '+' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::PlusAssign)
                } else {
                    self.add_token(TokenKind::Plus)
                }
            }
            '-' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::MinusAssign)
                } else if self.match_char('>') {
                    self.add_token(TokenKind::Arrow)
                } else {
                    self.add_token(TokenKind::Minus)
                }
            }
            '*' => {
                if self.match_char('*') {
                    self.add_token(TokenKind::StarStar)
                } else if self.match_char('=') {
                    self.add_token(TokenKind::StarAssign)
                } else {
                    self.add_token(TokenKind::Star)
                }
            }
            '/' => {
                if self.match_char('/') {
                    if self.match_char('=') {
                        self.add_token(TokenKind::SlashSlashAssign)
                    } else {
                        self.add_token(TokenKind::SlashSlash)
                    }
                } else if self.match_char('=') {
                    self.add_token(TokenKind::SlashAssign)
                } else {
                    self.add_token(TokenKind::Slash)
                }
            }
            '%' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::PercentAssign)
                } else {
                    self.add_token(TokenKind::Percent)
                }
            }
            '&' => self.add_token(TokenKind::BitAnd),
            '|' => self.add_token(TokenKind::BitOr),
            '^' => self.add_token(TokenKind::BitXor),
            '~' => self.add_token(TokenKind::BitNot),
            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::Eq)
                } else {
                    self.add_token(TokenKind::Assign)
                }
            }
            '<' => {
                if self.match_char('<') {
                    self.add_token(TokenKind::LShift)
                } else if self.match_char('=') {
                    self.add_token(TokenKind::LtEq)
                } else {
                    self.add_token(TokenKind::Lt)
                }
            }
            '>' => {
                if self.match_char('>') {
                    self.add_token(TokenKind::RShift)
                } else if self.match_char('=') {
                    self.add_token(TokenKind::GtEq)
                } else {
                    self.add_token(TokenKind::Gt)
                }
            }
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::NotEq)
                } else {
                    self.error_token(&format!(
                        "Unexpected character: {} on line {}",
                        c, self.line
                    ))
                }
            }
            '#' => {
                self.skip_comment();
                self.next_token()
            }
            '\n' => {
                self.line += 1;
                self.add_token(TokenKind::Newline)
            }
            '"' | '\'' => self.string(c, false, false, false),
            'b' => {
                if self.peek() == '"' || self.peek() == '\'' {
                    let quote_char = self.peek();
                    self.advance(); // consume the quote
                    self.string(quote_char, false, true, false) // is_fstring=false, is_bytes=true
                } else {
                    self.current = self.start;
                    self.identifier()
                }
            }
            'f' | 'F' => {
                if self.peek() == '"' || self.peek() == '\'' {
                    let quote_char = self.peek();
                    self.advance();
                    self.string(quote_char, true, false, false)
                } else {
                    self.current = self.start;
                    self.identifier()
                }
            }
            'r' | 'R' => {
                if self.peek() == '"' || self.peek() == '\'' {
                    let quote_char = self.peek();
                    self.advance(); // consume the quote
                    self.string(quote_char, false, false, true) // is_fstring=false, is_bytes=false, is_raw=true
                } else {
                    self.current = self.start;
                    self.identifier()
                }
            }
            _ if c.is_ascii_digit() => self.number(),
            _ if c.is_alphabetic() || c == '_' => self.identifier(),
            _ => self.error_token(&format!(
                "Unexpected character: {} on line {}",
                c, self.line
            )),
        }
    }
}
