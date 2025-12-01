#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate self as eldritchv2;

pub mod bindings;
mod lang;
pub mod repl;

// Re-export core types
pub use eldritch_macros::*;
pub use lang::ast::ForeignValue;
pub use lang::ast::Value;
pub use lang::interpreter::Interpreter;
pub use repl::Repl;

// Public API exports
pub use lang::conversion;
pub use lang::global_libs::register_lib;
