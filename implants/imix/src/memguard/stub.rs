//! No-op placeholder for `memguard` on non-Windows targets or when the
//! `mem-obfuscation` feature is disabled. Mirrors the real engine's public API
//! so call sites compile unconditionally.

/// No-op memory guard. The engine is only active on Windows with the
/// `mem-obfuscation` feature; everywhere else construction yields `None`.
pub struct MemGuard;

impl MemGuard {
    pub fn new() -> Option<Self> {
        None
    }

    pub fn encrypt(&mut self) -> bool {
        false
    }

    pub fn decrypt(&mut self) -> bool {
        false
    }

    pub fn add_exclusion(&mut self, _base: *const u8, _len: usize) -> bool {
        false
    }

    pub fn clear_dynamic_exclusions(&mut self) {}

    pub fn find_encrypted_page(&self, _addr: *const u8) -> Option<usize> {
        None
    }

    pub fn encrypted_count(&self) -> usize {
        0
    }
}
