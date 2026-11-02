use super::AesLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

pub mod decrypt_file_impl;
pub mod decrypt_impl;
pub mod encrypt_file_impl;
pub mod encrypt_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(AesLibrary)]
pub struct StdAesLibrary;

impl AesLibrary for StdAesLibrary {
    fn decrypt(&self, key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
        decrypt_impl::aes_decrypt(key, iv, data)
    }

    fn decrypt_file(&self, src: String, dst: String, key: String) -> Result<(), String> {
        decrypt_file_impl::aes_decrypt_file(src, dst, key)
    }

    fn encrypt(&self, key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
        encrypt_impl::aes_encrypt(key, iv, data)
    }

    fn encrypt_file(&self, src: String, dst: String, key: String) -> Result<(), String> {
        encrypt_file_impl::aes_encrypt_file(src, dst, key)
    }
}
