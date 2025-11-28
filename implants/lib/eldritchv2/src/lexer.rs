use super::token::Token;
use std::collections::VecDeque;

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

    fn add_token(&self, token: Token) -> Token {
        token
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
        if let Some(token) = Token::from_keyword(&value) {
            token
        } else {
            self.add_token(Token::Identifier(value))
        }
    }

    fn number(&mut self) -> Token {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        let value: String = self.source[self.start..self.current].iter().collect();
        let number: i64 = value.parse().unwrap_or(0);
        self.add_token(Token::Integer(number))
    }

    fn string(&mut self, quote_char: char, is_fstring: bool) -> Result<Token, String> {
        if is_fstring {
            self.start = self.current;
        } else {
            self.start += 1;
        }

        let mut fstring_tokens = Vec::new();
        let mut current_literal = String::new();

        while self.peek() != quote_char && !self.is_at_end() && self.peek() != '\n' {
            if self.peek() == '{' && is_fstring {
                if !current_literal.is_empty() {
                    fstring_tokens.push(Token::String(current_literal.clone()));
                    current_literal.clear();
                }
                self.advance();
                let expr_tokens = self.tokenize_fstring_expression()?;
                fstring_tokens.extend(expr_tokens);
                continue;
            }

            if self.peek() == '\\' {
                self.advance();
                match self.advance() {
                    'n' => current_literal.push('\n'),
                    't' => current_literal.push('\t'),
                    'r' => current_literal.push('\r'),
                    '\\' => current_literal.push('\\'),
                    '"' => current_literal.push('"'),
                    '\'' => current_literal.push('\''),
                    c => current_literal.push(c),
                }
            } else {
                current_literal.push(self.advance());
            }
        }

        if self.peek() != quote_char {
            return Err(format!("Unterminated string literal on line {}", self.line));
        }
        self.advance();

        if is_fstring {
            if !current_literal.is_empty() {
                fstring_tokens.push(Token::String(current_literal));
            }
            Ok(Token::FStringContent(fstring_tokens))
        } else {
            Ok(self.add_token(Token::String(current_literal)))
        }
    }

    fn tokenize_fstring_expression(&mut self) -> Result<Vec<Token>, String> {
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
            return Err(format!(
                "Unmatched '{{' in f-string expression starting at line {}",
                self.line
            ));
        }

        let end_of_expr = self.current;
        let expr_source: String = self.source[initial_start..end_of_expr].iter().collect();
        let mut expr_lexer = Lexer::new(expr_source);

        loop {
            let token = expr_lexer.next_token()?;
            match token {
                Token::Eof => break,
                Token::Newline | Token::Indent | Token::Dedent => continue,
                Token::String(s) => tokens.push(Token::String(s)),
                _ => tokens.push(token),
            }
        }
        self.advance();

        let mut final_tokens = vec![Token::LParen];
        final_tokens.extend(tokens);
        final_tokens.push(Token::RParen);
        Ok(final_tokens)
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            match self.next_token() {
                Ok(token) => match token {
                    Token::Eof => {
                        while *self.indent_stack.last().unwrap() > 0 {
                            self.indent_stack.pop();
                            tokens.push(Token::Dedent);
                        }
                        tokens.push(Token::Eof);
                        return Ok(tokens);
                    }
                    _ => tokens.push(token),
                },
                Err(e) => return Err(e),
            }
        }
    }

    fn next_token(&mut self) -> Result<Token, String> {
        if let Some(token) = self.pending_tokens.pop_front() {
            return Ok(token);
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
                return Ok(Token::Eof);
            }

            if self.peek() != '\n' {
                let current_indent = *self.indent_stack.last().unwrap();
                if indent_count > current_indent {
                    self.indent_stack.push(indent_count);
                    return Ok(Token::Indent);
                } else if indent_count < current_indent {
                    let mut dedents = Vec::new();
                    while *self.indent_stack.last().unwrap() > indent_count {
                        self.indent_stack.pop();
                        dedents.push(self.add_token(Token::Dedent));
                    }
                    if *self.indent_stack.last().unwrap() != indent_count {
                        return Err(format!("Inconsistent indentation on line {}", self.line));
                    }
                    self.current = self.start + indent_count;
                    if !dedents.is_empty() {
                        let first = dedents.remove(0);
                        for t in dedents {
                            self.pending_tokens.push_back(t);
                        }
                        return Ok(first);
                    }
                }
            } else {
                self.advance();
                self.line += 1;
                return self.next_token();
            }
        }

        while self.peek().is_whitespace() && self.peek() != '\n' {
            self.advance();
        }
        self.start = self.current;
        if self.is_at_end() {
            return Ok(Token::Eof);
        }

        let c = self.advance();

        match c {
            '(' => {
                self.nesting += 1;
                Ok(self.add_token(Token::LParen))
            }
            ')' => {
                if self.nesting > 0 {
                    self.nesting -= 1;
                }
                Ok(self.add_token(Token::RParen))
            }
            '[' => {
                self.nesting += 1;
                Ok(self.add_token(Token::LBracket))
            }
            ']' => {
                if self.nesting > 0 {
                    self.nesting -= 1;
                }
                Ok(self.add_token(Token::RBracket))
            }
            '{' => {
                self.nesting += 1;
                Ok(self.add_token(Token::LBrace))
            }
            '}' => {
                if self.nesting > 0 {
                    self.nesting -= 1;
                }
                Ok(self.add_token(Token::RBrace))
            }
            ',' => Ok(self.add_token(Token::Comma)),
            ':' => Ok(self.add_token(Token::Colon)),
            '.' => Ok(self.add_token(Token::Dot)),
            ';' => Ok(self.add_token(Token::Newline)),
            '+' => Ok(self.add_token(Token::Plus)),
            '-' => Ok(self.add_token(Token::Minus)),
            '*' => Ok(self.add_token(Token::Star)),
            '/' => Ok(self.add_token(Token::Slash)),
            '&' => Ok(self.add_token(Token::BitAnd)), // New
            '|' => Ok(self.add_token(Token::BitOr)),  // New
            '^' => Ok(self.add_token(Token::BitXor)), // New
            '~' => Ok(self.add_token(Token::BitNot)), // New
            '=' => Ok(if self.match_char('=') {
                self.add_token(Token::Eq)
            } else {
                self.add_token(Token::Assign)
            }),
            '<' => {
                // Updated for LShift
                if self.match_char('<') {
                    Ok(self.add_token(Token::LShift))
                } else if self.match_char('=') {
                    Ok(self.add_token(Token::LtEq))
                } else {
                    Ok(self.add_token(Token::Lt))
                }
            }
            '>' => {
                // Updated for RShift
                if self.match_char('>') {
                    Ok(self.add_token(Token::RShift))
                } else if self.match_char('=') {
                    Ok(self.add_token(Token::GtEq))
                } else {
                    Ok(self.add_token(Token::Gt))
                }
            }
            '!' => Ok(if self.match_char('=') {
                self.add_token(Token::NotEq)
            } else {
                return Err(format!("Unexpected character: {} on line {}", c, self.line));
            }),
            '#' => {
                self.skip_comment();
                self.next_token()
            }
            '\n' => {
                self.line += 1;
                Ok(self.add_token(Token::Newline))
            }
            '"' | '\'' => self.string(c, false),
            'f' | 'F' => {
                if self.peek() == '"' || self.peek() == '\'' {
                    let quote_char = self.peek();
                    self.advance();
                    self.string(quote_char, true)
                } else {
                    self.current = self.start;
                    Ok(self.identifier())
                }
            }
            _ if c.is_ascii_digit() => Ok(self.number()),
            _ if c.is_alphabetic() || c == '_' => Ok(self.identifier()),
            _ => Err(format!("Unexpected character: {} on line {}", c, self.line)),
        }
    }
}
