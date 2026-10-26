extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("utf16le")]
/// The `utf16le` library provides UTF-16LE encoding and decoding operations.
pub trait Utf16leLibrary {
    #[eldritch_method]
    /// Decodes UTF-16LE bytes into a UTF-8 string.
    fn decode(&self, content: Vec<u8>) -> Result<String, String>;

    #[eldritch_method]
    /// Encodes a UTF-8 string into UTF-16LE bytes.
    fn encode(&self, content: String) -> Result<Vec<u8>, String>;
}
