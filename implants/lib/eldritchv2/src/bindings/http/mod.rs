use eldritch_macros::{eldritch_library, eldritch_method};
use crate::lang::ast::Value;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[eldritch_library("http")]
pub trait HttpLibrary {
    #[eldritch_method]
    fn download(&self, url: String, path: String) -> Result<(), String>;

    #[eldritch_method]
    fn get(&self, url: String, headers: Option<BTreeMap<String, String>>) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    fn post(&self, url: String, body: Option<Vec<u8>>, headers: Option<BTreeMap<String, String>>) -> Result<BTreeMap<String, Value>, String>;
}
