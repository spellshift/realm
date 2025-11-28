// Export all modules publicly
pub mod ast;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod token;

// Export core types for easy access from consumers (like the REPL)
pub use ast::Value;
pub use interpreter::Interpreter;
