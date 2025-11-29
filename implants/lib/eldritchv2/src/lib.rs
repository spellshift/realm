#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod ast;
pub mod conversion;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod token;
pub mod macros;
pub mod repl;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

// Re-export core types
pub use ast::Value;
pub use interpreter::Interpreter;
pub use repl::Repl;

#[cfg(all(feature = "default_allocator", target_arch = "wasm32", not(feature = "std")))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// For non-wasm32 no_std builds, we might need an allocator if cdylib is built.
// But usually no_std on host is not the primary target.
// If we run 'cargo build' on host with default features (default_allocator),
// wee_alloc is NOT included (due to target cfg deps).
// So we might still fail on host if we don't have an allocator.
// However, adding a host-compatible no_std allocator is complex.
// We'll rely on the user knowing to use 'rlib' or 'std' for host builds if they need it.
// Or we can simple enable 'std' for host by default? No, user requested 'std' removal from default.

#[cfg(all(not(feature = "std"), target_arch = "wasm32"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
