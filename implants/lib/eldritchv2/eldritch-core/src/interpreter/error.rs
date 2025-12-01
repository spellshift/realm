use super::super::token::Span;
use alloc::string::{String, ToString};
use core::fmt;

#[derive(Debug)]
pub struct EldritchError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for EldritchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Runtime Error at line {}: {}",
            self.span.line, self.message
        )
    }
}

// Helper to create errors
pub fn runtime_error<T>(span: Span, msg: &str) -> Result<T, EldritchError> {
    Err(EldritchError {
        message: msg.to_string(),
        span,
    })
}
