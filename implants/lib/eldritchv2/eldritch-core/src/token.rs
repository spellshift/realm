use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize, // Byte index start
    pub end: usize,   // Byte index end
    pub line: usize,  // Line number (1-based)
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize) -> Self {
        Span { start, end, line }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
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
    Percent,
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
    SlashSlash,

    // Augmented assignments
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    PercentAssign,
    SlashSlashAssign,

    // Arrows
    Arrow,

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
    Bytes(Vec<u8>),
    Integer(i64),
    Float(f64),
    FStringContent(Vec<Token>),

    // Keywords
    Def,
    If,
    Elif,
    Else,
    Return,
    For,
    In,
    NotIn,
    True,
    False,
    None,
    And,
    Or,
    Not,
    Break,
    Continue,
    Pass,
    Lambda,

    // Structural
    Indent,
    Dedent,
    Eof,

    // Error
    Error(String),
}

impl TokenKind {
    pub fn from_keyword(s: &str) -> Option<TokenKind> {
        match s {
            "def" => Some(TokenKind::Def),
            "if" => Some(TokenKind::If),
            "elif" => Some(TokenKind::Elif),
            "else" => Some(TokenKind::Else),
            "return" => Some(TokenKind::Return),
            "for" => Some(TokenKind::For),
            "in" => Some(TokenKind::In),
            "True" => Some(TokenKind::True),
            "False" => Some(TokenKind::False),
            "None" => Some(TokenKind::None),
            "and" => Some(TokenKind::And),
            "or" => Some(TokenKind::Or),
            "not" => Some(TokenKind::Not),
            "break" => Some(TokenKind::Break),
            "continue" => Some(TokenKind::Continue),
            "pass" => Some(TokenKind::Pass),
            "lambda" => Some(TokenKind::Lambda),
            _ => None,
        }
    }
}
