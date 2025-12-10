pub mod core;
pub mod parser;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(test)]
mod tests;

pub use self::core::*;

extern crate alloc;
