extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("aes")]
/// The `aes` library provides AES-128-CBC encryption and decryption utilities.
///
/// Note: CBC mode provides confidentiality only (no integrity/authentication tag);
/// ciphertext is malleable unless paired with a MAC or an AEAD construction.
pub trait AesLibrary {
    #[eldritch_method]
    /// Decrypts data using AES-128 in CBC mode with PKCS#7 padding.
    ///
    /// `key` must be 16 bytes and `iv` must be 16 bytes; the IV is used to
    /// chain blocks (CBC), matching the `aes.encrypt` IV.
    fn decrypt(&self, key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    /// Decrypts a file using AES-128 in CBC mode with PKCS#7 padding.
    ///
    /// `key` must be 16 bytes (`Bytes` or `str`).
    fn decrypt_file(&self, src: String, dst: String, key: Vec<u8>) -> Result<(), String>;

    #[eldritch_method]
    /// Encrypts data using AES-128 in CBC mode with PKCS#7 padding.
    ///
    /// `key` must be 16 bytes and `iv` must be 16 bytes; the IV is used to
    /// chain blocks (CBC) and must be kept alongside the ciphertext to
    /// decrypt it later.
    fn encrypt(&self, key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    /// Encrypts a file using AES-128 in CBC mode with PKCS#7 padding and a random prepended IV.
    ///
    /// `key` must be 16 bytes (`Bytes` or `str`).
    fn encrypt_file(&self, src: String, dst: String, key: Vec<u8>) -> Result<(), String>;
}
