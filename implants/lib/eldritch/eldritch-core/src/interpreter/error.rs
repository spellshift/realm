use super::super::token::Span;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum EldritchErrorKind {
    SyntaxError,
    TypeError,
    NameError,
    IndexError,
    KeyError,
    AttributeError,
    ValueError,
    RuntimeError,
    RecursionError,
    ZeroDivisionError,
    ImportError,
    AssertionError,
}

impl fmt::Display for EldritchErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EldritchErrorKind::SyntaxError => write!(f, "SyntaxError"),
            EldritchErrorKind::TypeError => write!(f, "TypeError"),
            EldritchErrorKind::NameError => write!(f, "NameError"),
            EldritchErrorKind::IndexError => write!(f, "IndexError"),
            EldritchErrorKind::KeyError => write!(f, "KeyError"),
            EldritchErrorKind::AttributeError => write!(f, "AttributeError"),
            EldritchErrorKind::ValueError => write!(f, "ValueError"),
            EldritchErrorKind::RuntimeError => write!(f, "RuntimeError"),
            EldritchErrorKind::RecursionError => write!(f, "RecursionError"),
            EldritchErrorKind::ZeroDivisionError => write!(f, "ZeroDivisionError"),
            EldritchErrorKind::ImportError => write!(f, "ImportError"),
            EldritchErrorKind::AssertionError => write!(f, "AssertionError"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub name: String,
    pub filename: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct EldritchError {
    pub kind: EldritchErrorKind,
    pub message: String,
    pub span: Span,
    pub stack: Vec<StackFrame>,
}

impl EldritchError {
    pub fn new(kind: EldritchErrorKind, message: &str, span: Span) -> Self {
        EldritchError {
            kind,
            message: message.to_string(),
            span,
            stack: Vec::new(),
        }
    }

    pub fn with_stack(mut self, stack: Vec<StackFrame>) -> Self {
        self.stack = stack;
        self
    }
}

impl fmt::Display for EldritchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.stack.is_empty() {
            writeln!(f, "Traceback (most recent call last):")?;
            for frame in &self.stack {
                writeln!(
                    f,
                    "  File \"{}\", line {}, in {}",
                    frame.filename, frame.line, frame.name
                )?;
            }
        }

        // Also include the location of the error itself if not covered by stack
        // (Typically the last stack frame is the error location, but for syntax/runtime, the span is precise)
        // If stack is empty, we just print the error.

        // Note: The original format printed source line content.
        // We might want to handle that in the printer or Interpreter::format_error logic,
        // as we don't have source code access here easily.

        write!(f, "{}: {}", self.kind, self.message)?;

        // Add helpful advice based on error kind
        match self.kind {
            EldritchErrorKind::NameError => {
                if let Some(var_name) = self
                    .message
                    .strip_prefix("Undefined variable: '")
                    .and_then(|s| s.strip_suffix("'"))
                {
                    // Simple heuristic for advice
                    write!(f, "\nDid you mean to define '{}' or import it?", var_name)?;
                }
            }
            EldritchErrorKind::TypeError => {
                if self.message.contains("not iterable") {
                    write!(
                        f,
                        "\nEnsure you are iterating over a List, Tuple, Set, Dictionary, or String."
                    )?;
                } else if self.message.contains("not subscriptable") {
                    write!(
                        f,
                        "\nEnsure you are accessing an index on a List, Tuple, or Dictionary."
                    )?;
                }
            }
            EldritchErrorKind::KeyError => {
                write!(
                    f,
                    "\nThe key does not exist in the dictionary. Use .get() to avoid this error."
                )?;
            }
            _ => {}
        }

        Ok(())
    }
}

// Helper to create errors (Legacy support, mapped to RuntimeError or similar)
// We will deprecate this usage over time in favor of specific errors.
pub fn runtime_error<T>(span: Span, msg: &str) -> Result<T, EldritchError> {
    Err(EldritchError::new(
        EldritchErrorKind::RuntimeError,
        msg,
        span,
    ))
}

/// A type-safe error returned by native (builtin) functions and methods.
/// This replaces the old `Result<Value, String>` pattern where error kind
/// was encoded as a string prefix (e.g. "TypeError: ...").
#[derive(Debug, Clone)]
pub struct NativeError {
    pub kind: EldritchErrorKind,
    pub message: String,
}

impl NativeError {
    pub fn new(kind: EldritchErrorKind, message: impl Into<String>) -> Self {
        NativeError {
            kind,
            message: message.into(),
        }
    }

    pub fn type_error(message: impl Into<String>) -> Self {
        Self::new(EldritchErrorKind::TypeError, message)
    }

    pub fn value_error(message: impl Into<String>) -> Self {
        Self::new(EldritchErrorKind::ValueError, message)
    }

    pub fn runtime_error(message: impl Into<String>) -> Self {
        Self::new(EldritchErrorKind::RuntimeError, message)
    }

    pub fn index_error(message: impl Into<String>) -> Self {
        Self::new(EldritchErrorKind::IndexError, message)
    }

    pub fn key_error(message: impl Into<String>) -> Self {
        Self::new(EldritchErrorKind::KeyError, message)
    }

    /// Convert to a full EldritchError with span and stack information.
    pub fn into_eldritch_error(self, span: Span) -> EldritchError {
        EldritchError::new(self.kind, &self.message, span)
    }
}

impl fmt::Display for NativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

/// Conversion from String errors to NativeError for backward compatibility.
/// This parses the old "ErrorKind: message" prefix convention.
impl From<String> for NativeError {
    fn from(msg: String) -> Self {
        if let Some(rest) = msg.strip_prefix("TypeError: ") {
            NativeError::new(EldritchErrorKind::TypeError, rest)
        } else if let Some(rest) = msg.strip_prefix("ValueError: ") {
            NativeError::new(EldritchErrorKind::ValueError, rest)
        } else if let Some(rest) = msg.strip_prefix("IndexError: ") {
            NativeError::new(EldritchErrorKind::IndexError, rest)
        } else if let Some(rest) = msg.strip_prefix("KeyError: ") {
            NativeError::new(EldritchErrorKind::KeyError, rest)
        } else if let Some(rest) = msg.strip_prefix("AttributeError: ") {
            NativeError::new(EldritchErrorKind::AttributeError, rest)
        } else if let Some(rest) = msg.strip_prefix("NameError: ") {
            NativeError::new(EldritchErrorKind::NameError, rest)
        } else if let Some(rest) = msg.strip_prefix("AssertionError: ") {
            NativeError::new(EldritchErrorKind::AssertionError, rest)
        } else {
            NativeError::new(EldritchErrorKind::RuntimeError, msg)
        }
    }
}
