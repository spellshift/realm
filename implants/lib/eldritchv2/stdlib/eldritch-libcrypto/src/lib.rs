extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::{eldritch_library, eldritch_method};

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod std;

#[eldritch_library("crypto")]
pub trait CryptoLibrary {
    #[eldritch_method]
    fn aes_decrypt(&self, key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    fn aes_encrypt(&self, key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    fn md5(&self, data: Vec<u8>) -> Result<String, String>;

    #[eldritch_method]
    fn sha1(&self, data: Vec<u8>) -> Result<String, String>;

    #[eldritch_method]
    fn sha256(&self, data: Vec<u8>) -> Result<String, String>;

    #[eldritch_method]
    fn hash_file(&self, file: String, algo: String) -> Result<String, String>;
}
