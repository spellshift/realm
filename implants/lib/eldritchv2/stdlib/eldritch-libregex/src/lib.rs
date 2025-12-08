extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("regex")]
/// The `regex` library provides regular expression capabilities using Rust's `regex` crate syntax.
///
/// **Note**: Currently, it primarily supports a single capture group. Multi-group support might be limited.
pub trait RegexLibrary {
    #[eldritch_method]
    /// Returns all substrings matching the pattern in the haystack.
    ///
    /// If the pattern contains capture groups, returns the captured string for each match.
    ///
    /// **Parameters**
    /// - `haystack` (`str`): The string to search.
    /// - `pattern` (`str`): The regex pattern.
    ///
    /// **Returns**
    /// - `List<str>`: A list of matching strings.
    ///
    /// **Errors**
    /// - Returns an error string if the regex is invalid.
    fn match_all(&self, haystack: String, pattern: String) -> Result<Vec<String>, String>;

    #[eldritch_method("match")]
    /// Returns the first substring matching the pattern.
    ///
    /// **Parameters**
    /// - `haystack` (`str`): The string to search.
    /// - `pattern` (`str`): The regex pattern.
    ///
    /// **Returns**
    /// - `str`: The matching string.
    ///
    /// **Errors**
    /// - Returns an error string if no match is found or the regex is invalid.
    fn r#match(&self, haystack: String, pattern: String) -> Result<String, String>;

    #[eldritch_method]
    /// Replaces all occurrences of the pattern with the value.
    ///
    /// **Parameters**
    /// - `haystack` (`str`): The string to modify.
    /// - `pattern` (`str`): The regex pattern to match.
    /// - `value` (`str`): The replacement string.
    ///
    /// **Returns**
    /// - `str`: The modified string.
    ///
    /// **Errors**
    /// - Returns an error string if the regex is invalid.
    fn replace_all(
        &self,
        haystack: String,
        pattern: String,
        value: String,
    ) -> Result<String, String>;

    #[eldritch_method]
    /// Replaces the first occurrence of the pattern with the value.
    ///
    /// **Parameters**
    /// - `haystack` (`str`): The string to modify.
    /// - `pattern` (`str`): The regex pattern to match.
    /// - `value` (`str`): The replacement string.
    ///
    /// **Returns**
    /// - `str`: The modified string.
    ///
    /// **Errors**
    /// - Returns an error string if the regex is invalid.
    fn replace(&self, haystack: String, pattern: String, value: String) -> Result<String, String>;
}
