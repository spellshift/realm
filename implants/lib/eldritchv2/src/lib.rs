#![no_std]

extern crate alloc;

#[cfg(test)]
extern crate std;

mod ast;
mod evaluator;
mod lexer;
mod parser;
