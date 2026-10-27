#![allow(clippy::mutable_key_type)]
#![allow(unexpected_cfgs)]
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

#[eldritch_library("windows")]
/// The `windows` library provides Windows-specific capabilities including DLL injection/reflection and token manipulation.
pub trait WindowsLibrary {
    #[eldritch_method]
    /// Injects a DLL from disk into a remote process.
    ///
    /// **Parameters**
    /// - `dll_path` (`str`): Path to the DLL on disk.
    /// - `pid` (`int`): Target process ID.
    ///
    /// **Returns**
    /// - `None`
    fn dll_inject(&self, dll_path: String, pid: i64) -> Result<(), String>;

    #[eldritch_method]
    /// Reflectively injects a DLL from memory into a remote process.
    ///
    /// **Parameters**
    /// - `dll_bytes` (`List<int>`): Content of the DLL.
    /// - `pid` (`int`): Target process ID.
    /// - `function_name` (`str`): Exported function to call (for Imix DLLs, use `"lib_entry"`).
    ///
    /// **Returns**
    /// - `None`
    fn dll_reflect(
        &self,
        dll_bytes: Vec<u8>,
        pid: i64,
        function_name: String,
    ) -> Result<(), String>;

    #[eldritch_method]
    /// Impersonates another user by stealing a process token.
    ///
    /// **Parameters**
    /// - `pid` (`int`): Target process ID.
    ///
    /// **Returns**
    /// - `int`: Token store ID for later use with `use_token()`.
    fn impersonate(&self, pid: i64) -> Result<i64, String>;

    #[eldritch_method]
    /// Lists tokens in the global store, or enumerates a process token.
    ///
    /// **Parameters**
    /// - `pid` (`Option<int>`): Process ID to query, or None for stored tokens.
    ///
    /// **Returns**
    /// - `List<Dict>`: Token info.
    fn tokens(&self, pid: Option<i64>) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[eldritch_method]
    /// Activates a stored token by ID.
    ///
    /// **Parameters**
    /// - `id` (`int`): Token store ID (returned by `impersonate()` or other token functions).
    ///
    /// **Returns**
    /// - `bool`: True if token was activated.
    fn use_token(&self, id: i64) -> Result<bool, String>;
}
