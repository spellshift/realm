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

pub struct Lexer {}
