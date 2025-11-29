#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod ast;
pub mod conversion;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod token;
pub mod macros;
pub mod repl;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

// Re-export core types
pub use ast::Value;
pub use interpreter::Interpreter;
pub use repl::Repl;
