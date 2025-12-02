#![cfg_attr(feature = "no_std", no_std)]

extern crate alloc;
extern crate self as eldritch_core;

// Internal
mod ast;
mod global_libs;
mod interpreter;
mod lexer;
mod parser;
mod token;

// Re-export core types
pub use ast::{Environment, ForeignValue, Value};
pub use interpreter::{BufferPrinter, Interpreter, Printer, StdoutPrinter};
pub use lexer::Lexer;
pub use token::{Span, TokenKind};

// Public API exports
pub use global_libs::register_lib;
pub mod conversion;
