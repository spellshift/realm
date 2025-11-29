#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod ast;
pub mod conversion;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod token;
pub mod repl;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

// Re-export core types
pub use ast::Value;
pub use interpreter::Interpreter;
pub use repl::Repl;
pub use eldritch_macros::*;

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::string::String;
use spin::Mutex;

lazy_static::lazy_static! {
    static ref GLOBAL_LIBRARIES: Mutex<BTreeMap<String, Arc<dyn ast::ForeignValue>>> = Mutex::new(BTreeMap::new());
}

pub fn register_lib(val: impl ast::ForeignValue + 'static) {
    let mut libs = GLOBAL_LIBRARIES.lock();
    let name = val.type_name().to_string();
    libs.insert(name, Arc::new(val));
}

pub(crate) fn get_global_libraries() -> BTreeMap<String, Arc<dyn ast::ForeignValue>> {
    GLOBAL_LIBRARIES.lock().clone()
}
