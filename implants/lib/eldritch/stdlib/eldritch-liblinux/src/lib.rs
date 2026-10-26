extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("linux")]
/// The `linux` library provides Linux-specific system operations.
pub trait LinuxLibrary {
    #[eldritch_method]
    /// Calls the `uname` syscall and returns a dictionary of available system information.
    ///
    /// **Returns**
    /// - `Dict<str, str>`: Dictionary with `sysname`, `nodename`, `release`, `version`, `machine`, and `domainname`.
    fn uname(&self) -> Result<BTreeMap<String, String>, String>;
}
