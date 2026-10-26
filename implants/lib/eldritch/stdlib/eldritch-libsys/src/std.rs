use crate::SysLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_libfile::{FileLibrary, std::StdFileLibrary};
use eldritch_libnet::{NetLibrary, std::StdNetLibrary};
use eldritch_libprocess::{ProcessLibrary, std::StdProcessLibrary};
use eldritch_libuser::{UserLibrary, std::StdUserLibrary};
use eldritch_libwindows::{WindowsLibrary, std::StdWindowsLibrary};
use eldritch_macros::eldritch_library_impl;

mod exec_impl;
mod hostname_impl;
mod info_impl;
mod is_bsd_impl;
mod is_linux_impl;
mod is_macos_impl;
mod is_windows_impl;
mod shell_impl;
pub use eldritch_libwindows::std::tokens_impl;

#[derive(Debug, Default)]
#[eldritch_library_impl(SysLibrary)]
pub struct StdSysLibrary;

impl SysLibrary for StdSysLibrary {
    fn dll_inject(&self, dll_path: String, pid: i64) -> Result<(), String> {
        StdWindowsLibrary.dll_inject(dll_path, pid)
    }

    fn dll_reflect(
        &self,
        dll_bytes: Vec<u8>,
        pid: i64,
        function_name: String,
    ) -> Result<(), String> {
        StdWindowsLibrary.dll_reflect(dll_bytes, pid, function_name)
    }

    fn exec(
        &self,
        path: String,
        args: Vec<String>,
        disown: Option<bool>,
        env_vars: Option<BTreeMap<String, String>>,
        input: Option<String>,
    ) -> Result<BTreeMap<String, Value>, String> {
        exec_impl::exec(path, args, disown, env_vars, input).map_err(|e| e.to_string())
    }

    fn get_env(&self) -> Result<BTreeMap<String, String>, String> {
        StdProcessLibrary.env()
    }

    fn get_ip(&self) -> Result<Vec<BTreeMap<String, String>>, String> {
        StdNetLibrary.interfaces()
    }

    fn get_os(&self) -> Result<BTreeMap<String, String>, String> {
        self.info()
    }

    fn get_pid(&self) -> Result<i64, String> {
        StdProcessLibrary.pid()
    }

    fn get_reg(&self, path: String) -> Result<BTreeMap<String, String>, String> {
        StdWindowsLibrary.read_reg(path)
    }

    fn get_user(&self) -> Result<BTreeMap<String, Value>, String> {
        StdUserLibrary.info()
    }

    fn hostname(&self) -> Result<String, String> {
        hostname_impl::hostname().map_err(|e| e.to_string())
    }

    fn impersonate(&self, pid: i64) -> Result<i64, String> {
        StdWindowsLibrary.impersonate(pid)
    }

    fn info(&self) -> Result<BTreeMap<String, String>, String> {
        info_impl::get_os().map_err(|e| e.to_string())
    }

    fn is_bsd(&self) -> Result<bool, String> {
        is_bsd_impl::is_bsd().map_err(|e| e.to_string())
    }

    fn is_linux(&self) -> Result<bool, String> {
        is_linux_impl::is_linux().map_err(|e| e.to_string())
    }

    fn is_macos(&self) -> Result<bool, String> {
        is_macos_impl::is_macos().map_err(|e| e.to_string())
    }

    fn is_windows(&self) -> Result<bool, String> {
        is_windows_impl::is_windows().map_err(|e| e.to_string())
    }

    fn list_users(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
        StdUserLibrary.list()
    }

    fn shell(&self, cmd: String) -> Result<BTreeMap<String, Value>, String> {
        shell_impl::shell(cmd).map_err(|e| e.to_string())
    }

    fn tokens(&self, pid: Option<i64>) -> Result<Vec<BTreeMap<String, Value>>, String> {
        StdWindowsLibrary.tokens(pid)
    }

    fn use_token(&self, id: i64) -> Result<bool, String> {
        StdWindowsLibrary.use_token(id)
    }

    fn whereis(&self, bin: String) -> Result<String, String> {
        StdFileLibrary.which(bin)
    }

    fn write_reg(
        &self,
        path: String,
        regname: String,
        regtype: String,
        regvalue: Value,
    ) -> Result<bool, String> {
        StdWindowsLibrary.write_reg(path, regname, regtype, regvalue)
    }
}
