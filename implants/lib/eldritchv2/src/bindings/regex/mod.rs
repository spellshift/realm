use eldritch_macros::{eldritch_library, eldritch_method};
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[eldritch_library("regex")]
pub trait RegexLibrary {
    #[eldritch_method]
    fn match_all(&self, haystack: String, pattern: String) -> Result<Vec<String>, String>;

    #[eldritch_method]
    fn r#match(&self, haystack: String, pattern: String) -> Result<String, String>;

    #[eldritch_method]
    fn replace_all(&self, haystack: String, pattern: String, value: String) -> Result<String, String>;

    #[eldritch_method]
    fn replace(&self, haystack: String, pattern: String, value: String) -> Result<String, String>;
}
