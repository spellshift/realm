use crate::RegistryLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

mod read_impl;
#[cfg(target_os = "windows")]
pub mod reg_utils;
mod write_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(RegistryLibrary)]
pub struct StdRegistryLibrary;

impl RegistryLibrary for StdRegistryLibrary {
    fn read(&self, path: String) -> Result<BTreeMap<String, String>, String> {
        read_impl::read_reg(path).map_err(|e| e.to_string())
    }

    fn write(
        &self,
        path: String,
        regname: String,
        regtype: String,
        regvalue: Value,
    ) -> Result<bool, String> {
        write_impl::write_reg(path, regname, regtype, regvalue).map_err(|e| e.to_string())
    }
}
