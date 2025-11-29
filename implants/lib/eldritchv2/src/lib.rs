#![no_std]

extern crate alloc;

pub mod ast;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod token;

// Re-export core types
pub use ast::Value;
pub use interpreter::Interpreter;
