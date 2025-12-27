use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

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
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Assign => write!(f, "="),
            TokenKind::Newline => write!(f, "newline"),

            TokenKind::BitAnd => write!(f, "&"),
            TokenKind::BitOr => write!(f, "|"),
            TokenKind::BitXor => write!(f, "^"),
            TokenKind::BitNot => write!(f, "~"),
            TokenKind::LShift => write!(f, "<<"),
            TokenKind::RShift => write!(f, ">>"),
            TokenKind::StarStar => write!(f, "**"),
            TokenKind::SlashSlash => write!(f, "//"),

            TokenKind::PlusAssign => write!(f, "+="),
            TokenKind::MinusAssign => write!(f, "-="),
            TokenKind::StarAssign => write!(f, "*="),
            TokenKind::SlashAssign => write!(f, "/="),
            TokenKind::PercentAssign => write!(f, "%="),
            TokenKind::SlashSlashAssign => write!(f, "//="),

            TokenKind::Arrow => write!(f, "->"),

            TokenKind::Eq => write!(f, "=="),
            TokenKind::NotEq => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::LtEq => write!(f, "<="),
            TokenKind::GtEq => write!(f, ">="),

            TokenKind::Identifier(s) => write!(f, "identifier \"{}\"", s),
            TokenKind::String(s) => write!(f, "string {:?}", s),
            TokenKind::Bytes(_) => write!(f, "bytes"),
            TokenKind::Integer(i) => write!(f, "{}", i),
            TokenKind::Float(v) => write!(f, "{}", v),
            TokenKind::FStringContent(_) => write!(f, "f-string"),

            TokenKind::Def => write!(f, "def"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Elif => write!(f, "elif"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::NotIn => write!(f, "not in"),
            TokenKind::True => write!(f, "True"),
            TokenKind::False => write!(f, "False"),
            TokenKind::None => write!(f, "None"),
            TokenKind::And => write!(f, "and"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::Not => write!(f, "not"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Pass => write!(f, "pass"),
            TokenKind::Lambda => write!(f, "lambda"),

            TokenKind::Indent => write!(f, "indent"),
            TokenKind::Dedent => write!(f, "dedent"),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
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
