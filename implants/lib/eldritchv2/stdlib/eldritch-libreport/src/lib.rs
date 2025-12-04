extern crate alloc;
use eldritch_core::Value;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("report")]
pub trait ReportLibrary {
    #[eldritch_method]
    fn file(&self, path: String) -> Result<(), String>;

    #[eldritch_method]
    fn process_list(&self, list: Vec<BTreeMap<String, Value>>) -> Result<(), String>;

    #[eldritch_method]
    fn ssh_key(&self, username: String, key: String) -> Result<(), String>;

    #[eldritch_method]
    fn user_password(&self, username: String, password: String) -> Result<(), String>;
}
