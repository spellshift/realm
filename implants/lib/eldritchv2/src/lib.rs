#![cfg_attr(not(test), no_std)]

mod ast;
mod evaluator;
mod lexer;
mod parser;

// This module requires `std` for stdin/stdout. It won't be compiled in pure `no_std` builds.
#[cfg(feature = "std")]
pub mod repl;

extern crate alloc;
