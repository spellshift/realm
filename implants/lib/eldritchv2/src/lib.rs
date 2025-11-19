#![no_std]

extern crate alloc;

#[cfg(test)]
extern crate std;

pub mod ast;
pub mod evaluator;
pub mod lexer;
pub mod parser;
