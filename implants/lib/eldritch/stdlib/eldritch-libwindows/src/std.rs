use crate::WindowsLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

mod dll_inject_impl;
mod dll_reflect_impl;
mod impersonate_impl;
mod read_reg_impl;
mod reg_utils;
pub mod tokens_impl;
mod use_token_impl;
mod write_reg_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(WindowsLibrary)]
pub struct StdWindowsLibrary;

impl WindowsLibrary for StdWindowsLibrary {
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

    fn impersonate(&self, pid: i64) -> Result<i64, String> {
        impersonate_impl::impersonate(pid)
    }

    fn read_reg(&self, path: String) -> Result<BTreeMap<String, String>, String> {
        read_reg_impl::get_reg(path).map_err(|e| e.to_string())
    }

    fn tokens(&self, pid: Option<i64>) -> Result<Vec<BTreeMap<String, Value>>, String> {
        tokens_impl::tokens(pid)
    }

    fn use_token(&self, id: i64) -> Result<bool, String> {
        use_token_impl::use_token(id)
    }

    fn write_reg(
        &self,
        path: String,
        regname: String,
        regtype: String,
        regvalue: Value,
    ) -> Result<bool, String> {
        write_reg_impl::write_reg(path, regname, regtype, regvalue).map_err(|e| e.to_string())
    }
}
