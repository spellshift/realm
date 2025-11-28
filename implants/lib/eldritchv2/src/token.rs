#[derive(Debug, Clone, PartialEq)] // Added PartialEq derive for easier testing
pub enum Token {
    // Single-character tokens
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Dot, // Added Dot
    Minus,
    Plus,
    Star,
    Slash,
    Assign,
    Newline,

    // One or two character tokens
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq, // Added LtEq (<=) and GtEq (>=)

    // Literals
    Identifier(String),
    String(String),
    Integer(i64),
    FStringContent(Vec<Token>),

    // Keywords
    Def,
    If,
    Elif,
    Else,
    Return,
    For,
    In,
    True,
    False,
    None,
    And,
    Or,
    Not, // Added Logical Operators
    Break,
    Continue, // Added Loop Control

    // Structural
    Indent,
    Dedent,
    Eof,
}

impl Token {
    pub fn from_keyword(s: &str) -> Option<Token> {
        match s {
            "def" => Some(Token::Def),
            "if" => Some(Token::If),
            "elif" => Some(Token::Elif),
            "else" => Some(Token::Else),
            "return" => Some(Token::Return),
            "for" => Some(Token::For),
            "in" => Some(Token::In),
            "True" => Some(Token::True),
            "False" => Some(Token::False),
            "None" => Some(Token::None),
            "and" => Some(Token::And),
            "or" => Some(Token::Or),
            "not" => Some(Token::Not),
            "break" => Some(Token::Break),
            "continue" => Some(Token::Continue),
            _ => None,
        }
    }
}
