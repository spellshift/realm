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

    #[eldritch_method]
    /// Changes file attributes on a Linux filesystem (e.g. ext2/3/4, btrfs, xfs), behaving like
    /// the `chattr` command.
    ///
    /// **Parameters**
    /// - `path` (`str`): The file or directory to modify.
    /// - `attrs` (`str`): An attribute change string, matching `chattr` syntax:
    ///   - `+<attrs>`: Adds the given attributes (e.g. `"+i"` to set immutable).
    ///   - `-<attrs>`: Removes the given attributes (e.g. `"-i"` to unset immutable).
    ///   - `=<attrs>`: Sets the attributes exactly, clearing any others (e.g. `"=ai"`).
    ///   - Operators may be combined in a single string (e.g. `"+i-s"`).
    ///   - Supported attribute characters: `A S D a c m d e i j P s u t T C x F`.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the path doesn't exist, the attribute string is invalid, or
    ///   the underlying filesystem doesn't support the requested attributes.
    ///
    /// **Supported OS**
    /// - Linux only.
    fn chattr(&self, path: String, attrs: String) -> Result<(), String>;
}
