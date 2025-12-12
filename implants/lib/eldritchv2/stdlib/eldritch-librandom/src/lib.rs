extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("random")]
pub trait RandomLibrary {
    #[eldritch_method]
    fn bool(&self) -> Result<bool, String>;

    #[eldritch_method]
    fn bytes(&self, len: i64) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    fn int(&self, min: i64, max: i64) -> Result<i64, String>;

    #[eldritch_method]
    fn string(&self, len: i64, charset: Option<String>) -> Result<String, String>;

    #[eldritch_method]
    fn uuid(&self) -> Result<String, String>;
}
