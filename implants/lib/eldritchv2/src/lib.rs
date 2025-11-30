#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate self as eldritchv2;

mod lang;
pub mod repl;
pub mod bindings;

// Re-export core types
pub use lang::ast::Value;
pub use lang::ast::ForeignValue;
pub use lang::interpreter::Interpreter;
pub use repl::Repl;
pub use eldritch_macros::*;

// Public API exports
pub use lang::conversion;
pub use lang::global_libs::register_lib;
