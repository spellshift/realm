#![cfg_attr(feature = "no_std", no_std)]
#![allow(clippy::mutable_key_type)]

extern crate alloc;
extern crate self as eldritch_core;

// Internal
mod ast;
mod interpreter;
mod lexer;
mod parser;
mod token;

// Re-export core types
pub use ast::{Environment, ForeignValue, Value};
// Export AST nodes for LSP/Tooling
pub use ast::{Argument, Expr, ExprKind, FStringSegment, Param, Stmt, StmtKind};

pub use interpreter::{get_native_method_signature, BufferPrinter, Interpreter, Printer, StdoutPrinter};
pub use lexer::Lexer;
pub use token::{Span, TokenKind};

// Public API exports
pub mod conversion;
// Re-export introspection for macros and tests
pub use interpreter::introspection;

// Expose Parser for tests (restricted visibility not strictly enforced by test crate unless we re-export it)
// Tests integration via `tests/` directory sees `eldritch_core` as an external crate.
// So we must `pub use` it here for tests to see it.
// The `Lexer` and `TokenKind` are already re-exported.
pub use parser::Parser;
pub use interpreter::{MethodSignature, ParameterSignature};
