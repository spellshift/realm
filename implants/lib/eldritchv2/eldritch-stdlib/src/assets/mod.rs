use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[eldritch_library("assets")]
pub trait AssetsLibrary {
    #[eldritch_method]
    fn get(&self, name: String) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    fn list(&self) -> Result<Vec<String>, String>;
}
