#![allow(clippy::mutable_key_type)]
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

#[eldritch_library("http")]
pub trait HttpLibrary {
    #[eldritch_method]
    fn download(&self, url: String, path: String) -> Result<(), String>;

    #[eldritch_method]
    fn get(
        &self,
        url: String,
        headers: Option<BTreeMap<String, String>>,
    ) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    fn post(
        &self,
        url: String,
        body: Option<Vec<u8>>,
        headers: Option<BTreeMap<String, String>>,
    ) -> Result<BTreeMap<String, Value>, String>;
}
