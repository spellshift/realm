mod builtins;
mod core;
pub mod error;
mod eval;
mod exec;
pub mod introspection;
pub mod methods;
pub mod operations;
pub mod printer;

pub use self::core::Interpreter;
#[allow(unused_imports)]
pub use self::error::EldritchError;
#[allow(unused_imports)]
pub use self::error::EldritchErrorKind;
pub use self::methods::get_native_method_signature;
pub use self::printer::{BufferPrinter, Printer, StdoutPrinter};
pub mod signature;
pub use self::signature::{MethodSignature, ParameterSignature};
