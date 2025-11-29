use super::token::Token;
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

    fn string(
        &mut self,
        quote_char: char,
        is_fstring: bool,
        is_bytes: bool,
    ) -> Result<Token, String> {
        if is_fstring || is_bytes {
            self.start = self.current;
        } else {
            self.start += 1;
        }

        let mut fstring_tokens = Vec::new();
        let mut current_literal = String::new(); // For String or Bytes (as chars initially)

        // Check for triple quote
        let is_triple = if self.peek() == quote_char && self.peek_next() == quote_char {
            self.advance();
            self.advance();
            true
        } else {
            false
        };

        loop {
            if self.is_at_end() {
                return Err(format!("Unterminated string literal on line {}", self.line));
            }

            let c = self.peek();

            // Check end of string
            if c == quote_char {
                if is_triple {
                    if self.peek_next() == quote_char {
                        // Potential end of triple, need to check 3rd char
                        // We can't easily peek 2 ahead with this struct, but we can advance check
                        // Current at 1st quote.
                        if self.current + 2 < self.source.len()
                            && self.source[self.current + 2] == quote_char
                        {
                            self.advance(); // consume 1st
                            self.advance(); // consume 2nd
                            self.advance(); // consume 3rd
                            break;
                        }
                    }
                } else {
                    self.advance(); // consume quote
                    break;
                }
            }

            // Disallow unescaped newlines in single-quoted strings
            if c == '\n' {
                if !is_triple {
                    return Err(format!(
                        "Unterminated string literal (newline) on line {}",
                        self.line
                    ));
                }
                self.line += 1;
            }

            if c == '{' && is_fstring && !is_bytes {
                // F-strings can't be byte strings
                if !current_literal.is_empty() {
                    fstring_tokens.push(Token::String(current_literal.clone()));
                    current_literal.clear();
                }
                self.advance();
                let expr_tokens = self.tokenize_fstring_expression()?;
                fstring_tokens.extend(expr_tokens);
                continue;
            }

            if c == '\\' {
                self.advance();
                if self.is_at_end() {
                    return Err("Unterminated string literal".into());
                }
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
                    } // Line continuation
                    c => current_literal.push(c),
                }
            } else {
                current_literal.push(self.advance());
            }
        }

        if is_bytes {
            // Convert to bytes. In this simple impl, we just cast char to u8.
            // Real implementation would handle hex escapes \xNN etc.
            let bytes: Vec<u8> = current_literal.chars().map(|c| c as u8).collect();
            Ok(Token::Bytes(bytes))
        } else if is_fstring {
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
            '*' => {
                if self.match_char('*') {
                    Ok(self.add_token(Token::StarStar))
                } else {
                    Ok(self.add_token(Token::Star))
                }
            }
            '/' => Ok(self.add_token(Token::Slash)),
            '&' => Ok(self.add_token(Token::BitAnd)),
            '|' => Ok(self.add_token(Token::BitOr)),
            '^' => Ok(self.add_token(Token::BitXor)),
            '~' => Ok(self.add_token(Token::BitNot)),
            '=' => Ok(if self.match_char('=') {
                self.add_token(Token::Eq)
            } else {
                self.add_token(Token::Assign)
            }),
            '<' => {
                if self.match_char('<') {
                    Ok(self.add_token(Token::LShift))
                } else if self.match_char('=') {
                    Ok(self.add_token(Token::LtEq))
                } else {
                    Ok(self.add_token(Token::Lt))
                }
            }
            '>' => {
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
            '"' | '\'' => self.string(c, false, false),
            'b' => {
                if self.peek() == '"' || self.peek() == '\'' {
                    let quote_char = self.peek();
                    self.advance(); // consume the quote
                    self.string(quote_char, false, true) // is_fstring=false, is_bytes=true
                } else {
                    // Standard identifier starting with b
                    self.current = self.start;
                    Ok(self.identifier())
                }
            }
            'f' | 'F' => {
                if self.peek() == '"' || self.peek() == '\'' {
                    let quote_char = self.peek();
                    self.advance();
                    self.string(quote_char, true, false)
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
