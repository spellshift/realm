extern crate alloc;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
pub mod ssh;

pub mod browser;

pub use eldritch_repl::{Input, Repl, ReplAction};
