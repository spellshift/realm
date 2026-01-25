use super::CryptoLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

pub mod aes_decrypt_file_impl;
pub mod aes_decrypt_impl;
pub mod aes_encrypt_file_impl;
pub mod aes_encrypt_impl;
pub mod decode_b64_impl;
pub mod encode_b64_impl;
pub mod from_json_impl;
pub mod hash_file_impl;
pub mod is_json_impl;
pub mod md5_impl;
pub mod sha1_impl;
pub mod sha256_impl;
pub mod to_json_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(CryptoLibrary)]
pub struct StdCryptoLibrary;

impl CryptoLibrary for StdCryptoLibrary {
    fn aes_encrypt(&self, key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
        aes_encrypt_impl::aes_encrypt(key, iv, data)
    }

    fn aes_decrypt(&self, key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
        aes_decrypt_impl::aes_decrypt(key, iv, data)
    }

    fn aes_decrypt_file(&self, src: String, dst: String, key: String) -> Result<(), String> {
        aes_decrypt_file_impl::aes_decrypt_file(src, dst, key)
    }

    fn aes_encrypt_file(&self, src: String, dst: String, key: String) -> Result<(), String> {
        aes_encrypt_file_impl::aes_encrypt_file(src, dst, key)
    }

    fn md5(&self, data: Vec<u8>) -> Result<String, String> {
        md5_impl::md5(data)
    }

    fn sha1(&self, data: Vec<u8>) -> Result<String, String> {
        sha1_impl::sha1(data)
    }

    fn sha256(&self, data: Vec<u8>) -> Result<String, String> {
        sha256_impl::sha256(data)
    }

    fn hash_file(&self, file: String, algo: String) -> Result<String, String> {
        hash_file_impl::hash_file(file, algo)
    }

    fn encode_b64(&self, content: String, encode_type: Option<String>) -> Result<String, String> {
        encode_b64_impl::encode_b64(content, encode_type)
    }

    fn decode_b64(&self, content: String, encode_type: Option<String>) -> Result<String, String> {
        decode_b64_impl::decode_b64(content, encode_type)
    }

    fn is_json(&self, content: String) -> Result<bool, String> {
        is_json_impl::is_json(content)
    }

    fn from_json(&self, content: String) -> Result<Value, String> {
        from_json_impl::from_json(content)
    }

    fn to_json(&self, content: Value) -> Result<String, String> {
        to_json_impl::to_json(content)
    }
}
