extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;
#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("assets")]
pub trait AssetsLibrary {
    #[eldritch_method]
    fn read_binary(&self, name: String) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    fn read(&self, name: String) -> Result<String, String>;

    #[eldritch_method]
    fn copy(&self, src: String, dest: String) -> Result<(), String>;

    #[eldritch_method]
    fn list(&self) -> Result<Vec<String>, String>;
}
