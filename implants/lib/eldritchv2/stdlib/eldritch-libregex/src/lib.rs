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
    fn match_all(&self, haystack: String, pattern: String) -> Result<Vec<String>, String>;

    #[eldritch_method("match")]
    fn r#match(&self, haystack: String, pattern: String) -> Result<String, String>;

    #[eldritch_method]
    fn replace_all(
        &self,
        haystack: String,
        pattern: String,
        value: String,
    ) -> Result<String, String>;

    #[eldritch_method]
    fn replace(&self, haystack: String, pattern: String, value: String) -> Result<String, String>;
}
