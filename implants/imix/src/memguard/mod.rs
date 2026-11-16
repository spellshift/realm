//! Core memory obfuscation engine.
//!
//! The real engine is only compiled on Windows with the `mem-obfuscation`
//! feature. On every other target (or with the feature disabled) a no-op stub
//! with the same public API is provided so call sites compile unconditionally.

#[cfg(all(windows, feature = "mem-obfuscation"))]
#[path = "engine.rs"]
mod engine;

#[cfg(all(windows, feature = "mem-obfuscation"))]
pub use engine::MemGuard;

#[cfg(not(all(windows, feature = "mem-obfuscation")))]
#[path = "stub.rs"]
mod stub;

#[cfg(not(all(windows, feature = "mem-obfuscation")))]
pub use stub::MemGuard;
