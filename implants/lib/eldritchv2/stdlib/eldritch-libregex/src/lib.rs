extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("regex")]
pub trait RegexLibrary {
    #[eldritch_method]
    fn match_all(&self, pattern: String, haystack: String) -> Result<Vec<String>, String>;

    #[eldritch_method("match")]
    fn r#match(&self, pattern: String, haystack: String) -> Result<String, String>;

    #[eldritch_method]
    fn replace_all(
        &self,
        pattern: String,
        haystack: String,
        value: String,
    ) -> Result<String, String>;

    #[eldritch_method]
    fn replace(&self, pattern: String, haystack: String, value: String) -> Result<String, String>;
}
