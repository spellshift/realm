extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("hash")]
/// The `hash` library provides cryptographic hashing functions for byte buffers and files.
pub trait HashLibrary {
    #[eldritch_method]
    /// Computes the cryptographic hash of a file using the specified algorithm ("md5", "sha1", "sha256", "sha512").
    fn file(&self, file: String, algo: String) -> Result<String, String>;

    #[eldritch_method]
    /// Computes the MD5 hash of the given bytes.
    fn md5(&self, data: Vec<u8>) -> Result<String, String>;

    #[eldritch_method]
    /// Computes the SHA1 hash of the given bytes.
    fn sha1(&self, data: Vec<u8>) -> Result<String, String>;

    #[eldritch_method]
    /// Computes the SHA256 hash of the given bytes.
    fn sha256(&self, data: Vec<u8>) -> Result<String, String>;
}
