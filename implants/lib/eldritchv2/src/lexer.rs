use super::token::Token;

pub struct Lexer {
    source: Vec<char>,
    start: usize,
    current: usize,
    line: usize,
    indent_stack: Vec<usize>,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        // Add a newline and an EOF sentinel to the end of the source for simplified parsing
        let mut chars: Vec<char> = source.chars().collect();
        chars.push('\n');

        Lexer {
            source: chars,
            start: 0,
            current: 0,
            line: 1,
            indent_stack: vec![0], // Start with 0 indentation
        }
    }

    // --- Core Helpers ---

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

    // --- Specialized Tokenizers ---

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
        let number: i64 = value.parse().unwrap_or(0); // Assuming only integers for simplicity
        self.add_token(Token::Integer(number))
    }

    // Updated: Now accepts is_fstring explicitly instead of inferring from quote_char
    fn string(&mut self, quote_char: char, is_fstring: bool) -> Result<Token, String> {
        // Skip the initial quote if it's a regular string, or 'f"' / 'f'' if it's an f-string
        if is_fstring {
            self.start = self.current;
        } else {
            self.start += 1;
        }

        let mut fstring_tokens = Vec::new();
        let mut current_literal = String::new();

        while self.peek() != quote_char && !self.is_at_end() && self.peek() != '\n' {
            if self.peek() == '{' && is_fstring {
                // If we encounter a '{' in an f-string:

                // 1. Push any accumulated literal text as a String token
                if !current_literal.is_empty() {
                    fstring_tokens.push(Token::String(current_literal.clone()));
                    current_literal.clear();
                } else if fstring_tokens.is_empty() {
                    // If it's the very start (f"{x}"), we might want an empty string token
                    // to maintain [Literal, Expr, Literal] structure, but the Parser
                    // handles arbitrary streams now, so we can skip empty leading literals.
                }

                // 2. Parse the internal expression:
                self.advance(); // consume '{'
                let expr_tokens = self.tokenize_fstring_expression()?;
                fstring_tokens.extend(expr_tokens);

                // 3. Continue the loop. tokenize_fstring_expression consumes the '}',
                // so we continue parsing the next character (literal or another '{')
                continue;
            }

            // Handle escape sequences
            if self.peek() == '\\' {
                self.advance(); // Consume '\'
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

        self.advance(); // consume closing quote

        if is_fstring {
            // Push the final literal part if it exists
            if !current_literal.is_empty() {
                fstring_tokens.push(Token::String(current_literal));
            }
            // Return the macro-token containing the stream
            Ok(Token::FStringContent(fstring_tokens))
        } else {
            // Treat as a normal string literal
            Ok(self.add_token(Token::String(current_literal)))
        }
    }

    // Helper function to tokenize the content inside an f-string expression { ... }
    fn tokenize_fstring_expression(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        let initial_start = self.current;
        let mut nesting_level = 1;

        // Find the matching '}' for the expression
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

        // Create a temporary lexer for the content between initial_start and end_of_expr
        let expr_source: String = self.source[initial_start..end_of_expr].iter().collect();
        let mut expr_lexer = Lexer::new(expr_source);

        // Collect tokens from the expression
        loop {
            // Note: The internal lexer will automatically add an Eof, which we stop at.
            let token = expr_lexer.next_token()?;
            match token {
                Token::Eof => break,
                // Ignore artifacts of the temporary Lexer creation (it adds a newline by default)
                Token::Newline | Token::Indent | Token::Dedent => continue,
                // Replace 'normal' string tokens inside the expression with a single String token.
                Token::String(s) => tokens.push(Token::String(s)),
                _ => tokens.push(token),
            }
        }

        self.advance(); // Consume the final '}'

        // The token stream for the expression will look like:
        // [LParen, token1, token2, ..., RParen]
        // The parser will expect LParen/RParen instead of { / }
        let mut final_tokens = vec![Token::LParen];
        final_tokens.extend(tokens);
        final_tokens.push(Token::RParen);

        Ok(final_tokens)
    }

    // --- Main Token Scan ---

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            match self.next_token() {
                Ok(token) => {
                    match token {
                        Token::Eof => {
                            // Ensure final dedents are added before EOF
                            while *self.indent_stack.last().unwrap() > 0 {
                                self.indent_stack.pop();
                                tokens.push(Token::Dedent);
                            }
                            tokens.push(Token::Eof);
                            return Ok(tokens);
                        }
                        // Fix: Don't flatten FStringContent. The Parser expects it as a single unit.
                        _ => tokens.push(token),
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    fn next_token(&mut self) -> Result<Token, String> {
        self.start = self.current;

        // Handle structural indentation at the start of a line
        if self.start > 0 && self.source[self.start - 1] == '\n' {
            // Newline tokens are already processed on the previous line.
            // Check for current indentation.
            let mut indent_count = 0;
            while self.peek().is_whitespace() && self.peek() != '\n' {
                self.advance();
                indent_count += 1;
            }

            if self.is_at_end() {
                return Ok(Token::Eof);
            }

            // If the line is not just whitespace
            if self.peek() != '\n' {
                let current_indent = *self.indent_stack.last().unwrap();

                if indent_count > current_indent {
                    self.indent_stack.push(indent_count);
                    return Ok(Token::Indent);
                } else if indent_count < current_indent {
                    let mut tokens = Vec::new();
                    while *self.indent_stack.last().unwrap() > indent_count {
                        self.indent_stack.pop();
                        tokens.push(self.add_token(Token::Dedent));
                    }
                    if *self.indent_stack.last().unwrap() != indent_count {
                        return Err(format!("Inconsistent indentation on line {}", self.line));
                    }
                    // Since we can only return one token, we return the first one and adjust
                    // the current pointer back to re-scan for the rest of the tokens on the next loop.
                    self.current = self.start + indent_count;
                    if let Some(token) = tokens.get(0).cloned() {
                        return Ok(token);
                    }
                }
            } else {
                // If the line is empty (just a newline), consume the newline and restart
                self.advance();
                self.line += 1;
                return self.next_token();
            }
        }

        // Consume whitespace before scanning the actual token
        while self.peek().is_whitespace() && self.peek() != '\n' {
            self.advance();
        }

        self.start = self.current;

        if self.is_at_end() {
            return Ok(Token::Eof);
        }

        let c = self.advance();

        match c {
            '(' => Ok(self.add_token(Token::LParen)),
            ')' => Ok(self.add_token(Token::RParen)),
            '[' => Ok(self.add_token(Token::LBracket)),
            ']' => Ok(self.add_token(Token::RBracket)),
            '{' => Ok(self.add_token(Token::LBrace)),
            '}' => Ok(self.add_token(Token::RBrace)),
            ',' => Ok(self.add_token(Token::Comma)),
            ':' => Ok(self.add_token(Token::Colon)),
            '+' => Ok(self.add_token(Token::Plus)),
            '-' => Ok(self.add_token(Token::Minus)),
            '*' => Ok(self.add_token(Token::Star)),
            '/' => Ok(self.add_token(Token::Slash)),
            '=' => Ok(if self.match_char('=') {
                self.add_token(Token::Eq)
            } else {
                self.add_token(Token::Assign)
            }),
            '<' => Ok(self.add_token(Token::Lt)),
            '>' => Ok(self.add_token(Token::Gt)),
            '!' => Ok(if self.match_char('=') {
                self.add_token(Token::NotEq)
            } else {
                return Err(format!("Unexpected character: {} on line {}", c, self.line));
            }),
            '\n' => {
                self.line += 1;
                Ok(self.add_token(Token::Newline))
            }
            // Pass false for normal strings
            '"' | '\'' => self.string(c, false),
            'f' | 'F' => {
                if self.peek() == '"' || self.peek() == '\'' {
                    let quote_char = self.peek();
                    self.advance(); // consume the quote
                                    // Pass true for f-strings
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
