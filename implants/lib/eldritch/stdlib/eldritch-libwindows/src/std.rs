use crate::WindowsLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

mod dll_inject_impl;
mod dll_reflect_impl;
mod enable_privilege_impl;
mod impersonate_impl;
pub mod tokens_impl;
mod use_token_impl;

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

    fn enable_privilege(&self, privilege: String) -> Result<bool, String> {
        enable_privilege_impl::enable_privilege(privilege)
    }

    fn impersonate(&self, pid: i64) -> Result<i64, String> {
        impersonate_impl::impersonate(pid)
    }

    fn tokens(&self, pid: Option<i64>) -> Result<Vec<BTreeMap<String, Value>>, String> {
        tokens_impl::tokens(pid)
    }

    fn use_token(&self, id: i64) -> Result<bool, String> {
        use_token_impl::use_token(id)
    }
}
