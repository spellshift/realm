use eldritch_macros::{eldritch_library, eldritch_method};
use crate::lang::ast::Value;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "std")]
pub mod std;

#[eldritch_library("process")]
pub trait ProcessLibrary {
    #[eldritch_method]
    fn info(&self, pid: Option<i64>) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    fn kill(&self, pid: i64) -> Result<(), String>;

    #[eldritch_method]
    fn list(&self) -> Result<Vec<BTreeMap<String, Value>>, String>;

    #[eldritch_method]
    fn name(&self, pid: i64) -> Result<String, String>;

    #[eldritch_method]
    fn netstat(&self) -> Result<Vec<BTreeMap<String, Value>>, String>;
}
