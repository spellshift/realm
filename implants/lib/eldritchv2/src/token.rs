use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
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
    Dot,
    Minus,
    Plus,
    Star,
    Slash,
    Assign,
    Newline,

    // Bitwise operators
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    LShift,
    RShift,
    StarStar,

    // One or two character tokens
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,

    // Literals
    Identifier(String),
    String(String),
    Bytes(Vec<u8>), // New: Byte string literal
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
    Not,
    Break,
    Continue,

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
