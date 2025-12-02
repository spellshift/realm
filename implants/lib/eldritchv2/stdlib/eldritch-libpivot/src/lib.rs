extern crate alloc;
use eldritch_core::Value;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[eldritch_library("pivot")]
pub trait PivotLibrary {
    #[eldritch_method]
    fn list(&self) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[eldritch_method]
    fn start_tcp(&self, bind_addr: String) -> Result<String, String>;

    #[eldritch_method]
    fn stop(&self, id: String) -> Result<(), String>;
}
