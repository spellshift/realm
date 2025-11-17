use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Special tokens
    Illegal,
    Eof,

    // Identifiers and literals
    Identifier(String),
    Integer(i64),
    String(String),

    // Operators
    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,
    LessThan,
    GreaterThan,
    Equal,
    NotEqual,

    // Delimiters
    Comma,
    Semicolon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,

    // Keywords
    Function,
    True,
    False,
    If,
    Else,
    Return,
    For,
    In,
}

#[derive(Debug)]
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    read_position: usize,
    ch: char,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer {
            input: input.chars().collect(),
            position: 0,
            read_position: 0,
            ch: '\0',
        };
        lexer.read_char();
        lexer
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.read_position];
        }
        self.position = self.read_position;
        self.read_position += 1;
    }
    fn peek_char(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position]
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.ch {
            '"' => Token::String(self.read_string()),
            '=' => {
                if self.peek_char() == '=' {
                    // "==" → Equal
                    self.read_char(); // consume second '='
                    Token::Equal
                } else {
                    Token::Assign
                }
            }
            ';' => Token::Semicolon,
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            ',' => Token::Comma,
            '+' => Token::Plus,
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            '[' => Token::LeftBracket,
            ']' => Token::RightBracket,
            '!' => {
                if self.peek_char() == '=' {
                    // "!=" → NotEqual
                    self.read_char(); // consume second '='
                    Token::NotEqual
                } else {
                    Token::Bang
                }
            }
            '-' => Token::Minus,
            '/' => Token::Slash,
            '*' => Token::Asterisk,
            '<' => Token::LessThan,
            '>' => Token::GreaterThan,
            '\0' => Token::Eof,
            _ => {
                if is_letter(self.ch) {
                    let identifier = self.read_identifier();
                    return match identifier.as_str() {
                        "fn" => Token::Function,
                        "true" => Token::True,
                        "false" => Token::False,
                        "if" => Token::If,
                        "else" => Token::Else,
                        "return" => Token::Return,
                        "for" => Token::For,
                        "in" => Token::In,
                        _ => Token::Identifier(identifier),
                    };
                } else if self.ch.is_ascii_digit() {
                    return Token::Integer(self.read_number());
                } else {
                    Token::Illegal
                }
            }
        };

        self.read_char();
        token
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_whitespace() {
            self.read_char();
        }
    }

    fn read_identifier(&mut self) -> String {
        let position = self.position;
        while is_letter(self.ch) {
            self.read_char();
        }
        self.input[position..self.position].iter().collect()
    }

    fn read_number(&mut self) -> i64 {
        let position = self.position;
        while self.ch.is_ascii_digit() {
            self.read_char();
        }
        self.input[position..self.position]
            .iter()
            .collect::<String>()
            .parse()
            .unwrap()
    }

    fn read_string(&mut self) -> String {
        // We are currently on the opening quote `"`.
        // Move to the first character inside the string.
        self.read_char();
        let start = self.position;

        // Read until we hit the closing quote or end-of-input.
        while self.ch != '"' && self.ch != '\0' {
            self.read_char();
        }

        // At this point, self.ch is either '"' (closing) or '\0'.
        // Collect chars from start up to (but NOT including) the closing quote.
        self.input[start..self.position].iter().collect::<String>()
    }
}

fn is_letter(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_next_token() {
        let input = r#"let five = 5;
let ten = 10;

let add = fn(x, y) {
  x + y;
};

let result = add(five, ten);
!-/ *5;
5 < 10 > 5;

if (5 < 10) {
	return true;
} else {
	return false;
}
"#;

        let tokens = vec![
            Token::Identifier("five".to_string()),
            Token::Assign,
            Token::Integer(5),
            Token::Semicolon,
            Token::Identifier("ten".to_string()),
            Token::Assign,
            Token::Integer(10),
            Token::Semicolon,
            Token::Identifier("add".to_string()),
            Token::Assign,
            Token::Function,
            Token::LeftParen,
            Token::Identifier("x".to_string()),
            Token::Comma,
            Token::Identifier("y".to_string()),
            Token::RightParen,
            Token::LeftBrace,
            Token::Identifier("x".to_string()),
            Token::Plus,
            Token::Identifier("y".to_string()),
            Token::Semicolon,
            Token::RightBrace,
            Token::Semicolon,
            Token::Identifier("result".to_string()),
            Token::Assign,
            Token::Identifier("add".to_string()),
            Token::LeftParen,
            Token::Identifier("five".to_string()),
            Token::Comma,
            Token::Identifier("ten".to_string()),
            Token::RightParen,
            Token::Semicolon,
            Token::Bang,
            Token::Minus,
            Token::Slash,
            Token::Asterisk,
            Token::Integer(5),
            Token::Semicolon,
            Token::Integer(5),
            Token::LessThan,
            Token::Integer(10),
            Token::GreaterThan,
            Token::Integer(5),
            Token::Semicolon,
            Token::If,
            Token::LeftParen,
            Token::Integer(5),
            Token::LessThan,
            Token::Integer(10),
            Token::RightParen,
            Token::LeftBrace,
            Token::Return,
            Token::True,
            Token::Semicolon,
            Token::RightBrace,
            Token::Else,
            Token::LeftBrace,
            Token::Return,
            Token::False,
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];

        let mut lexer = Lexer::new(input);
        for token in tokens {
            assert_eq!(lexer.next_token(), token);
        }
    }
}
