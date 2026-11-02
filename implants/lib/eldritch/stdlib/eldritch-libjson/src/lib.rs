extern crate alloc;
use alloc::string::String;
use eldritch_core::Value;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("json")]
/// The `json` library provides JSON encoding, decoding, and validation operations.
pub trait JsonLibrary {
    #[eldritch_method]
    /// Parses a JSON string into an Eldritch value.
    fn decode(&self, content: String) -> Result<Value, String>;

    #[eldritch_method]
    /// Serializes an Eldritch value into a JSON string.
    fn encode(&self, content: Value) -> Result<String, String>;

    #[eldritch_method]
    /// Checks if a string is valid JSON.
    fn is_valid(&self, content: String) -> Result<bool, String>;
}
