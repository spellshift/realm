extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("aes")]
/// The `aes` library provides AES encryption and decryption utilities.
pub trait AesLibrary {
    #[eldritch_method]
    /// Decrypts data using AES (ECB/block mode with PKCS#7 padding).
    fn decrypt(&self, key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    /// Decrypts a file using AES.
    fn decrypt_file(&self, src: String, dst: String, key: String) -> Result<(), String>;

    #[eldritch_method]
    /// Encrypts data using AES (ECB/block mode with PKCS#7 padding).
    fn encrypt(&self, key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    /// Encrypts a file using AES.
    fn encrypt_file(&self, src: String, dst: String, key: String) -> Result<(), String>;
}
