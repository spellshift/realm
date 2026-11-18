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
#[cfg(all(windows, feature = "mem-obfuscation"))]
pub(crate) use engine::exclude_current_thread;

// QA / Phase-4 test hook: simulate a watchdog crash mid-decrypt so the VEH path
// can be exercised deliberately. Only present in test/print_debug builds.
// `allow(unused_imports)` is intentional: in a non-test print_debug build this
// export is deliberately available for Phase-4 QA but not referenced by imix.
#[cfg(all(
    windows,
    feature = "mem-obfuscation",
    any(test, feature = "print_debug")
))]
#[allow(unused_imports)]
pub use engine::debug_simulate_crash_partial_decrypt;

// The watchdog drives the encrypt/decrypt dormancy cycle from a dedicated
// `std::thread`. It is only built on Windows with the feature enabled; every
// other target gets inert fallbacks so `sleep_until_next_cycle` compiles and
// always falls through to the normal `tokio::time::sleep` path.
#[cfg(all(windows, feature = "mem-obfuscation"))]
mod watchdog;

#[cfg(all(windows, feature = "mem-obfuscation"))]
pub use watchdog::{MIN_ENCRYPT_INTERVAL_SECS, engage_memguard_sleep, min_encrypt_interval_secs};

#[cfg(not(all(windows, feature = "mem-obfuscation")))]
#[path = "stub.rs"]
mod stub;

#[cfg(not(all(windows, feature = "mem-obfuscation")))]
pub use stub::MemGuard;

#[cfg(not(all(windows, feature = "mem-obfuscation")))]
pub const MIN_ENCRYPT_INTERVAL_SECS: f64 = 10.0;

#[cfg(not(all(windows, feature = "mem-obfuscation")))]
pub fn min_encrypt_interval_secs() -> f64 {
    MIN_ENCRYPT_INTERVAL_SECS
}

#[cfg(not(all(windows, feature = "mem-obfuscation")))]
pub async fn engage_memguard_sleep(_interval_secs: f64) -> bool {
    false
}

// Serializes tests that create the `MemGuard` singleton so the engine's
// round-trip test and the watchdog integration test never race on the
// process-wide guard.
#[cfg(all(test, windows, feature = "mem-obfuscation"))]
pub(crate) static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
