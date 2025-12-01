#![cfg_attr(no_std, no_std)]

extern crate alloc;
extern crate self as eldritch_core;

// Internal
mod interpreter;
mod ast;
mod lexer;
mod parser;
mod token;
mod global_libs;

// Re-export core types
pub use ast::{Value, ForeignValue};
pub use interpreter::Interpreter;

// Public API exports
pub use global_libs::register_lib;
pub mod conversion;
