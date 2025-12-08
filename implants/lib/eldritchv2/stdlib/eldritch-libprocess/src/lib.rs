extern crate alloc;
use eldritch_core::Value;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("process")]
/// The `process` library allows interaction with system processes.
///
/// It supports:
/// - Listing running processes.
/// - Retrieving process details (info, name).
/// - Killing processes.
/// - Inspecting network connections (netstat).
pub trait ProcessLibrary {
    #[eldritch_method]
    /// Returns detailed information about a specific process.
    ///
    /// **Parameters**
    /// - `pid` (`Option<int>`): The process ID to query. If `None`, returns info for the current agent process.
    ///
    /// **Returns**
    /// - `Dict`: Dictionary with process details (pid, name, cmd, exe, environ, cwd, memory_usage, user, etc.).
    ///
    /// **Errors**
    /// - Returns an error string if the process is not found or cannot be accessed.
    fn info(&self, pid: Option<i64>) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    /// Terminates a process by its ID.
    ///
    /// **Parameters**
    /// - `pid` (`int`): The process ID to kill.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the process cannot be killed (e.g., permission denied).
    fn kill(&self, pid: i64) -> Result<(), String>;

    #[eldritch_method]
    /// Lists all currently running processes.
    ///
    /// **Returns**
    /// - `List<Dict>`: A list of process dictionaries containing `pid`, `ppid`, `name`, `path`, `username`, `command`, `cwd`, etc.
    ///
    /// **Errors**
    /// - Returns an error string if the process list cannot be retrieved.
    fn list(&self) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[eldritch_method]
    /// Returns the name of a process given its ID.
    ///
    /// **Parameters**
    /// - `pid` (`int`): The process ID.
    ///
    /// **Returns**
    /// - `str`: The process name.
    ///
    /// **Errors**
    /// - Returns an error string if the process is not found.
    fn name(&self, pid: i64) -> Result<String, String>;

    #[eldritch_method]
    /// Returns a list of active network connections (TCP/UDP/Unix).
    ///
    /// **Returns**
    /// - `List<Dict>`: A list of connection details including socket type, local/remote address/port, and associated PID.
    ///
    /// **Errors**
    /// - Returns an error string if network information cannot be retrieved.
    fn netstat(&self) -> Result<Vec<BTreeMap<String, Value>>, String>;
}
