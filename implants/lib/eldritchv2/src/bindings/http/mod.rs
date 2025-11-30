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
    fn request(&self, method: String, url: String, headers: Option<BTreeMap<String, String>>, body: Option<Vec<u8>>) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    fn upload(&self, url: String, path: String) -> Result<(), String>;
}
