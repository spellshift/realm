extern crate alloc;
use alloc::string::String;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("time")]
pub trait TimeLibrary {
    #[eldritch_method]
    fn format_to_epoch(&self, input: String, format: String) -> Result<i64, String>;

    #[eldritch_method]
    fn format_to_readable(&self, input: i64, format: String) -> Result<String, String>;

    #[eldritch_method]
    fn now(&self) -> Result<i64, String>;

    #[eldritch_method]
    fn sleep(&self, secs: i64) -> Result<(), String>;
}
