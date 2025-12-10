mod builtins;
mod core;
pub mod error;
mod eval;
pub mod operations;
mod exec;
mod methods;
pub mod printer;
pub mod introspection;

pub use self::core::Interpreter;
pub use self::printer::{BufferPrinter, Printer, StdoutPrinter};
pub use self::error::{EldritchError, EldritchErrorKind};
