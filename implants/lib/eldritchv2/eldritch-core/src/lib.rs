#![cfg_attr(feature = "no_std", no_std)]
#![allow(clippy::mutable_key_type)]

extern crate alloc;
extern crate self as eldritch_core;

// Internal
pub mod analysis;
mod ast;
mod interpreter;
mod lexer;
mod parser;
mod token;

// Re-export core types
pub use analysis::find_node_at_offset;
pub use ast::{
    Argument, Environment, ExprKind, FStringSegment, ForeignValue, Param, Stmt, StmtKind, Value,
};
pub use interpreter::{BufferPrinter, Interpreter, Printer, StdoutPrinter};
pub use lexer::Lexer;
pub use token::{Span, Token, TokenKind};

// Public API exports
pub mod conversion;
// Re-export introspection for macros and tests
pub use interpreter::introspection;

// Expose Parser for tests (restricted visibility not strictly enforced by test crate unless we re-export it)
// Tests integration via `tests/` directory sees `eldritch_core` as an external crate.
// So we must `pub use` it here for tests to see it.
// The `Lexer` and `TokenKind` are already re-exported.
pub use parser::Parser;
