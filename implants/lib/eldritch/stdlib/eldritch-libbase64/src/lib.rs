extern crate alloc;
use alloc::string::String;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("base64")]
/// The `base64` library provides Base64 encoding and decoding operations.
pub trait Base64Library {
    #[eldritch_method]
    /// Decodes a Base64 string.
    ///
    /// `encode_type` can be "STANDARD", "STANDARD_NO_PAD", "URL_SAFE", or "URL_SAFE_NO_PAD".
    fn decode(&self, content: String, encode_type: Option<String>) -> Result<String, String>;

    #[eldritch_method]
    /// Encodes a string into Base64.
    ///
    /// `encode_type` can be "STANDARD", "STANDARD_NO_PAD", "URL_SAFE", or "URL_SAFE_NO_PAD".
    fn encode(&self, content: String, encode_type: Option<String>) -> Result<String, String>;
}
