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

#[eldritch_library("sys")]
/// The `sys` library provides general system interaction capabilities.
///
/// It supports:
/// - Process execution (`exec`, `shell`).
/// - System information (`get_os`, `get_ip`, `get_user`, `hostname`).
/// - Registry operations (Windows).
/// - DLL injection and reflection.
/// - Environment variable access.
pub trait SysLibrary {
    #[eldritch_method]
    /// Injects a DLL from disk into a remote process.
    ///
    /// **Parameters**
    /// - `dll_path` (`str`): Path to the DLL on disk.
    /// - `pid` (`int`): Target process ID.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if injection fails.
    fn dll_inject(&self, dll_path: String, pid: i64) -> Result<(), String>;

    #[eldritch_method]
    /// Reflectively injects a DLL from memory into a remote process.
    ///
    /// **Parameters**
    /// - `dll_bytes` (`List<int>`): Content of the DLL.
    /// - `pid` (`int`): Target process ID.
    /// - `function_name` (`str`): Exported function to call.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if injection fails.
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
    /// Returns the current process's environment variables.
    ///
    /// **Returns**
    /// - `Dict<str, str>`: Map of environment variables.
    fn get_env(&self) -> Result<BTreeMap<String, String>, String>;

    #[eldritch_method]
    /// Returns network interface information.
    ///
    /// **Returns**
    /// - `List<Dict>`: List of interfaces with `name` and `ip`.
    fn get_ip(&self) -> Result<Vec<BTreeMap<String, String>>, String>;

    #[eldritch_method]
    /// Returns information about the operating system.
    ///
    /// **Returns**
    /// - `Dict`: Details like `arch`, `distro`, `platform`.
    fn get_os(&self) -> Result<BTreeMap<String, String>, String>;

    #[eldritch_method]
    /// Returns the current process ID.
    ///
    /// **Returns**
    /// - `int`: The PID.
    fn get_pid(&self) -> Result<i64, String>;

    #[eldritch_method]
    /// Reads values from the Windows Registry.
    ///
    /// **Parameters**
    /// - `reghive` (`str`): The registry hive (e.g., "HKEY_LOCAL_MACHINE").
    /// - `regpath` (`str`): The registry path.
    ///
    /// **Returns**
    /// - `Dict<str, str>`: A dictionary of registry keys and values.
    fn get_reg(&self, reghive: String, regpath: String)
    -> Result<BTreeMap<String, String>, String>;

    #[eldritch_method]
    /// Returns information about the current user.
    ///
    /// **Returns**
    /// - `Dict`: User details (uid, gid, name, groups).
    fn get_user(&self) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    /// Returns the system hostname.
    ///
    /// **Returns**
    /// - `str`: The hostname.
    fn hostname(&self) -> Result<String, String>;

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
    /// Executes a command via the system shell (`/bin/sh` or `cmd.exe`).
    ///
    /// **Parameters**
    /// - `cmd` (`str`): The command string to execute.
    ///
    /// **Returns**
    /// - `Dict`: Output containing `stdout`, `stderr`, and `status`.
    fn shell(&self, cmd: String) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    /// Writes a hex value to the Windows Registry.
    ///
    /// **Parameters**
    /// - `reghive` (`str`)
    /// - `regpath` (`str`)
    /// - `regname` (`str`)
    /// - `regtype` (`str`): e.g., "REG_BINARY".
    /// - `regvalue` (`str`): Hex string.
    ///
    /// **Returns**
    /// - `bool`: True on success.
    fn write_reg_hex(
        &self,
        reghive: String,
        regpath: String,
        regname: String,
        regtype: String,
        regvalue: String,
    ) -> Result<bool, String>;

    #[eldritch_method]
    /// Writes an integer value to the Windows Registry.
    ///
    /// **Parameters**
    /// - `reghive` (`str`)
    /// - `regpath` (`str`)
    /// - `regname` (`str`)
    /// - `regtype` (`str`): e.g., "REG_DWORD".
    /// - `regvalue` (`int`)
    ///
    /// **Returns**
    /// - `bool`: True on success.
    fn write_reg_int(
        &self,
        reghive: String,
        regpath: String,
        regname: String,
        regtype: String,
        regvalue: i64,
    ) -> Result<bool, String>;

    #[eldritch_method]
    /// Writes a string value to the Windows Registry.
    ///
    /// **Parameters**
    /// - `reghive` (`str`)
    /// - `regpath` (`str`)
    /// - `regname` (`str`)
    /// - `regtype` (`str`): e.g., "REG_SZ".
    /// - `regvalue` (`str`)
    ///
    /// **Returns**
    /// - `bool`: True on success.
    fn write_reg_str(
        &self,
        reghive: String,
        regpath: String,
        regname: String,
        regtype: String,
        regvalue: String,
    ) -> Result<bool, String>;
}
