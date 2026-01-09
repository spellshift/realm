use super::RegexLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

pub mod match_all_impl;
pub mod match_impl;
pub mod replace_all_impl;
pub mod replace_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(RegexLibrary)]
pub struct StdRegexLibrary;

impl RegexLibrary for StdRegexLibrary {
    fn match_all(&self, haystack: String, pattern: String) -> Result<Vec<String>, String> {
        match_all_impl::match_all(haystack, pattern)
    }

    fn r#match(&self, haystack: String, pattern: String) -> Result<String, String> {
        match_impl::r#match(haystack, pattern)
    }

    fn replace_all(
        &self,
        haystack: String,
        pattern: String,
        value: String,
    ) -> Result<String, String> {
        replace_all_impl::replace_all(haystack, pattern, value)
    }

    fn replace(&self, haystack: String, pattern: String, value: String) -> Result<String, String> {
        replace_impl::replace(haystack, pattern, value)
    }
}
