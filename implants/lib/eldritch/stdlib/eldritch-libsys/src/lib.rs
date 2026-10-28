#![allow(clippy::mutable_key_type)]
#![allow(unexpected_cfgs)]
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::{eldritch_deprecated, eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("sys")]
/// The `sys` library provides general system interaction capabilities.
///
/// It supports:
/// - Process execution (`exec`, `shell`).
/// - System information (`info`, `hostname`, `is_linux`, `is_windows`, `is_macos`, `is_bsd`).
pub trait SysLibrary {
    #[eldritch_method]
    #[eldritch_deprecated(
        "sys.dll_inject has been deprecated and will be removed in a future version of eldritch. Please use windows.dll_inject instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Injects a DLL from disk into a remote process.
    fn dll_inject(&self, dll_path: String, pid: i64) -> Result<(), String>;

    #[eldritch_method]
    #[eldritch_deprecated(
        "sys.dll_reflect has been deprecated and will be removed in a future version of eldritch. Please use windows.dll_reflect instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Reflectively injects a DLL from memory into a remote process.
    fn dll_reflect(
        &self,
        dll_bytes: Vec<u8>,
        pid: i64,
        function_name: String,
    ) -> Result<(), String>;

    #[eldritch_method]
    /// Executes a program directly (without a shell).
    ///
    /// **Parameters**
    /// - `path` (`str`): Path to the executable.
    /// - `args` (`List<str>`): List of arguments.
    /// - `disown` (`Option<bool>`): If `True`, runs in background/detached.
    /// - `env_vars` (`Option<Dict<str, str>>`): Environment variables to set.
    ///
    /// **Returns**
    /// - `Dict`: Output containing `stdout`, `stderr`, and `status` (exit code).
    fn exec(
        &self,
        path: String,
        args: Vec<String>,
        disown: Option<bool>,
        env_vars: Option<BTreeMap<String, String>>,
        input: Option<String>,
    ) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    #[eldritch_deprecated(
        "sys.get_env has been deprecated and will be removed in a future version of eldritch. Please use process.env instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Returns the current process's environment variables.
    fn get_env(&self) -> Result<BTreeMap<String, String>, String>;

    #[eldritch_method]
    #[eldritch_deprecated(
        "sys.get_ip has been deprecated and will be removed in a future version of eldritch. Please use net.interfaces instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Returns network interface information.
    fn get_ip(&self) -> Result<Vec<BTreeMap<String, String>>, String>;

    #[eldritch_method]
    #[eldritch_deprecated(
        "sys.get_os has been deprecated and will be removed in a future version of eldritch. Please use sys.info instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Returns information about the operating system.
    fn get_os(&self) -> Result<BTreeMap<String, String>, String>;

    #[eldritch_method]
    #[eldritch_deprecated(
        "sys.get_pid has been deprecated and will be removed in a future version of eldritch. Please use process.pid instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Returns the current process ID.
    fn get_pid(&self) -> Result<i64, String>;

    #[eldritch_method]
    #[eldritch_deprecated(
        "sys.get_reg has been deprecated and will be removed in a future version of eldritch. Please use registry.read instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Reads values from the Windows Registry.
    fn get_reg(&self, path: String) -> Result<BTreeMap<String, String>, String>;

    #[eldritch_method]
    #[eldritch_deprecated(
        "sys.get_user has been deprecated and will be removed in a future version of eldritch. Please use user.info instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Returns information about the current user.
    fn get_user(&self) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    /// Returns the system hostname.
    ///
    /// **Returns**
    /// - `str`: The hostname.
    fn hostname(&self) -> Result<String, String>;

    #[eldritch_method]
    #[eldritch_deprecated(
        "sys.impersonate has been deprecated and will be removed in a future version of eldritch. Please use windows.impersonate instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Impersonates another user by stealing a process token.
    fn impersonate(&self, pid: i64) -> Result<i64, String>;

    #[eldritch_method]
    /// Returns information about the operating system.
    ///
    /// **Returns**
    /// - `Dict`: Details like `arch`, `desktop_env`, `distro`, `platform`.
    fn info(&self) -> Result<BTreeMap<String, String>, String>;

    #[eldritch_method]
    /// Checks if the OS is BSD.
    ///
    /// **Returns**
    /// - `bool`: True if BSD.
    fn is_bsd(&self) -> Result<bool, String>;

    #[eldritch_method]
    /// Checks if the OS is Linux.
    ///
    /// **Returns**
    /// - `bool`: True if Linux.
    fn is_linux(&self) -> Result<bool, String>;

    #[eldritch_method]
    /// Checks if the OS is macOS.
    ///
    /// **Returns**
    /// - `bool`: True if macOS.
    fn is_macos(&self) -> Result<bool, String>;

    #[eldritch_method]
    /// Checks if the OS is Windows.
    ///
    /// **Returns**
    /// - `bool`: True if Windows.
    fn is_windows(&self) -> Result<bool, String>;

    #[eldritch_method]
    #[eldritch_deprecated(
        "sys.list_users has been deprecated and will be removed in a future version of eldritch. Please use user.list instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Lists all user accounts on the system.
    fn list_users(&self) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[eldritch_method]
    /// Executes a command via the system shell (`/bin/sh` or `cmd.exe`).
    ///
    /// **Parameters**
    /// - `cmd` (`str`): The command string to execute.
    ///
    /// **Returns**
    /// - `Dict`: Output containing `stdout`, `stderr`, and `status`.
    fn shell(&self, cmd: String) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    #[eldritch_deprecated(
        "sys.tokens has been deprecated and will be removed in a future version of eldritch. Please use windows.tokens instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Lists tokens in the global store, or enumerates a process token.
    fn tokens(&self, pid: Option<i64>) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[eldritch_method]
    #[eldritch_deprecated(
        "sys.use_token has been deprecated and will be removed in a future version of eldritch. Please use windows.use_token instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Activates a stored token by ID.
    fn use_token(&self, id: i64) -> Result<bool, String>;

    #[eldritch_method]
    #[eldritch_deprecated(
        "sys.whereis has been deprecated and will be removed in a future version of eldritch. Please use file.which instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Looks up the full path in PATH
    fn whereis(&self, bin: String) -> Result<String, String>;

    #[eldritch_method]
    #[eldritch_deprecated(
        "sys.write_reg has been deprecated and will be removed in a future version of eldritch. Please use registry.write instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    )]
    /// Writes a value to the Windows Registry.
    fn write_reg(
        &self,
        path: String,
        regname: String,
        regtype: String,
        regvalue: Value,
    ) -> Result<bool, String>;
}
