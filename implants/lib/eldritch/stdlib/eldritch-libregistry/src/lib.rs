#![no_std]
extern crate alloc;

#[cfg(feature = "fake_bindings")]
pub mod fake;
#[cfg(feature = "stdlib")]
pub mod std;

use alloc::collections::BTreeMap;
use alloc::string::String;
use eldritch_core::Value;
use eldritch_macros::{eldritch_library, eldritch_method};

#[eldritch_library("registry")]
pub trait RegistryLibrary {
    /// Read registry key values.
    #[eldritch_method]
    fn read(&self, path: String) -> Result<BTreeMap<String, String>, String>;

    /// Write a registry value.
    #[eldritch_method]
    fn write(
        &self,
        path: String,
        regname: String,
        regtype: String,
        regvalue: Value,
    ) -> Result<bool, String>;
}
