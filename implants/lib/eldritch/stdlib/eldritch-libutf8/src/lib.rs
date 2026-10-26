extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("utf8")]
/// The `utf8` library provides UTF-8 encoding and decoding operations.
pub trait Utf8Library {
    #[eldritch_method]
    /// Decodes UTF-8 bytes into a string.
    fn decode(&self, content: Vec<u8>) -> Result<String, String>;

    #[eldritch_method]
    /// Encodes a string into UTF-8 bytes.
    fn encode(&self, content: String) -> Result<Vec<u8>, String>;
}
