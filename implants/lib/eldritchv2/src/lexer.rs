use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Spanned<T> {
    pub token: T,
    pub start: Location,
    pub end: Location,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    Def,
    If,
    Elif,
    Else,
    For,
    In,
    Return,
    Break,
    Continue,
    Pass,
    Load,
    Lambda,
    And,
    Or,
    Not,

    // Literals
    Identifier(String),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),

    // Operators
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    FloorDivAssign,
    ModAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    LShiftAssign,
    RShiftAssign,

    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    Plus,
    Minus,
    Star,
    Slash,
    FloorDiv,
    Percent,
    Ampersand,
    Pipe,
    Caret,
    LShift,
    RShift,
    Tilde,

    // Delimiters
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Colon,
    Semicolon,
    StarStar,

    // Whitespace
    Newline,
    Indent,
    Outdent,

    // Control
    Eof,
    Error(String),
}

pub struct Lexer<'a> {
    chars: core::iter::Peekable<core::str::Chars<'a>>,
    indent_stack: Vec<usize>,
    token_buffer: Vec<Token>,
    at_line_start: bool,
    eof: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.chars().peekable(),
            indent_stack: vec![0],
            token_buffer: Vec::new(),
            at_line_start: true,
            eof: false,
        }
    }

    fn advance(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn read_identifier(&mut self, first: char) -> String {
        let mut ident = String::new();
        ident.push(first);
        while let Some(&c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        ident
    }

    fn read_number(&mut self, first: char) -> Token {
        let mut num_str = String::new();
        num_str.push(first);
        while let Some(&c) = self.peek() {
            if c.is_digit(10) {
                num_str.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        Token::Int(num_str.parse().unwrap())
    }

    fn handle_indentation(&mut self) -> Option<Token> {
        let mut indent_level = 0;
        loop {
            match self.peek() {
                Some(' ') => {
                    self.advance();
                    indent_level += 1;
                }
                Some('\t') => {
                    self.advance();
                    indent_level = (indent_level / 4 + 1) * 4;
                }
                _ => break,
            }
        }

        if let Some('/') = self.peek() {
            self.advance();
            if self.peek() == Some(&'/') {
                self.advance();
                while let Some(c) = self.peek() {
                    if *c == '\n' {
                        break;
                    }
                    self.advance();
                }
            }
        }

        if self.peek().is_none() {
             while self.indent_stack.len() > 1 {
                self.indent_stack.pop();
                self.token_buffer.push(Token::Outdent);
            }
            self.token_buffer.reverse();
            return self.token_buffer.pop()
        }

        if self.peek() == Some(&'\n') {
            self.advance();
            self.at_line_start = true;
            return Some(Token::Newline);
        }

        self.at_line_start = false;
        let last_indent = *self.indent_stack.last().unwrap();
        if indent_level > last_indent {
            self.indent_stack.push(indent_level);
            Some(Token::Indent)
        } else if indent_level < last_indent {
            while indent_level < *self.indent_stack.last().unwrap() {
                self.indent_stack.pop();
                self.token_buffer.push(Token::Outdent);
            }
            self.token_buffer.reverse();
            self.token_buffer.pop()
        } else {
            None
        }
    }

    fn next_token_internal(&mut self) -> Option<Token> {
        if !self.token_buffer.is_empty() {
            return self.token_buffer.pop();
        }

        if self.at_line_start {
            if let Some(token) = self.handle_indentation() {
                return Some(token);
            }
        }

        while let Some(&c) = self.peek() {
            if c.is_whitespace() && c != '\n' {
                self.advance();
            } else {
                break;
            }
        }

        if let Some(c) = self.advance() {
            let token = match c {
                '\n' => {
                    self.at_line_start = true;
                    Token::Newline
                }
                '/' => {
                    if self.peek() == Some(&'/') {
                        self.advance();
                        while let Some(c) = self.peek() {
                            if *c == '\n' {
                                break;
                            }
                            self.advance();
                        }
                        return self.next();
                    } else {
                        Token::Slash
                    }
                }
                '(' => Token::LParen,
                ')' => Token::RParen,
                ',' => Token::Comma,
                ':' => Token::Colon,
                '>' => Token::Greater,
                '<' => Token::Less,
                '=' => Token::Assign,
                c if c.is_alphabetic() => {
                    let ident = self.read_identifier(c);
                    match ident.as_str() {
                        "def" => Token::Def,
                        "for" => Token::For,
                        "in" => Token::In,
                        "if" => Token::If,
                        "elif" => Token::Elif,
                        "else" => Token::Else,
                        "return" => Token::Return,
                        "true" => Token::Identifier("true".to_string()),
                        "false" => Token::Identifier("false".to_string()),
                        _ => Token::Identifier(ident),
                    }
                }
                c if c.is_digit(10) => self.read_number(c),
                _ => Token::Error(format!("Unexpected char {}", c)),
            };
            Some(token)
        } else {
            if self.eof {
                return None;
            }
            self.eof = true;
            while self.indent_stack.len() > 1 {
                self.indent_stack.pop();
                self.token_buffer.push(Token::Outdent);
            }
            if !self.token_buffer.is_empty() {
                self.token_buffer.reverse();
                return self.token_buffer.pop();
            }
            None
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token_internal()
    }
}

#[cfg(test)]
mod tests;
