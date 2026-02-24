use crate::SysLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

mod change_user_password_impl;
mod dll_inject_impl;
mod dll_reflect_impl;
mod exec_impl;
mod get_env_impl;
mod get_ip_impl;
mod get_os_impl;
mod get_pid_impl;
mod get_reg_impl;
mod get_user_impl;
mod hostname_impl;
mod is_bsd_impl;
mod is_linux_impl;
mod is_macos_impl;
mod is_windows_impl;
mod list_users_impl;
mod shell_impl;
mod write_reg_hex_impl;
mod write_reg_int_impl;
mod write_reg_str_impl;

#[derive(Debug)]
#[eldritch_library_impl(SysLibrary)]
pub struct StdSysLibrary;

impl SysLibrary for StdSysLibrary {
    fn change_user_password(&self, username: String, password: String) -> Result<bool, String> {
        change_user_password_impl::change_user_password(username, password)
            .map_err(|e| e.to_string())
    }

    fn dll_inject(&self, dll_path: String, pid: i64) -> Result<(), String> {
        dll_inject_impl::dll_inject(dll_path, pid as u32).map_err(|e| e.to_string())
    }

    fn dll_reflect(
        &self,
        dll_bytes: Vec<u8>,
        pid: i64,
        function_name: String,
    ) -> Result<(), String> {
        dll_reflect_impl::dll_reflect(dll_bytes, pid as u32, function_name)
            .map_err(|e| e.to_string())
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
        get_env_impl::get_env().map_err(|e| e.to_string())
    }

    fn get_ip(&self) -> Result<Vec<BTreeMap<String, String>>, String> {
        get_ip_impl::get_ip().map_err(|e| e.to_string())
    }

    fn get_os(&self) -> Result<BTreeMap<String, String>, String> {
        get_os_impl::get_os().map_err(|e| e.to_string())
    }

    fn get_pid(&self) -> Result<i64, String> {
        get_pid_impl::get_pid()
            .map(|pid| pid as i64)
            .map_err(|e| e.to_string())
    }

    fn get_reg(
        &self,
        reghive: String,
        regpath: String,
    ) -> Result<BTreeMap<String, String>, String> {
        get_reg_impl::get_reg(reghive, regpath).map_err(|e| e.to_string())
    }

    fn get_user(&self) -> Result<BTreeMap<String, Value>, String> {
        get_user_impl::get_user().map_err(|e| e.to_string())
    }

    fn hostname(&self) -> Result<String, String> {
        hostname_impl::hostname().map_err(|e| e.to_string())
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
        list_users_impl::list_users().map_err(|e| e.to_string())
    }

    fn shell(&self, cmd: String) -> Result<BTreeMap<String, Value>, String> {
        shell_impl::shell(cmd).map_err(|e| e.to_string())
    }

    fn write_reg_hex(
        &self,
        reghive: String,
        regpath: String,
        regname: String,
        regtype: String,
        regvalue: String,
    ) -> Result<bool, String> {
        write_reg_hex_impl::write_reg_hex(reghive, regpath, regname, regtype, regvalue)
            .map_err(|e| e.to_string())
    }

    fn write_reg_int(
        &self,
        reghive: String,
        regpath: String,
        regname: String,
        regtype: String,
        regvalue: i64,
    ) -> Result<bool, String> {
        write_reg_int_impl::write_reg_int(reghive, regpath, regname, regtype, regvalue as u32)
            .map_err(|e| e.to_string())
    }

    fn write_reg_str(
        &self,
        reghive: String,
        regpath: String,
        regname: String,
        regtype: String,
        regvalue: String,
    ) -> Result<bool, String> {
        write_reg_str_impl::write_reg_str(reghive, regpath, regname, regtype, regvalue)
            .map_err(|e| e.to_string())
    }
}
