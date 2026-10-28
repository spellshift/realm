use super::CryptoLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::{Value, call_bound_method};
use eldritch_libaes::{AesLibrary, fake::AesLibraryFake};
use eldritch_libbase64::{Base64Library, fake::Base64LibraryFake};
use eldritch_libhash::{HashLibrary, fake::HashLibraryFake};
use eldritch_libjson::{JsonLibrary, fake::JsonLibraryFake};
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(CryptoLibrary)]
pub struct CryptoLibraryFake;

impl CryptoLibrary for CryptoLibraryFake {
    fn aes_decrypt(&self, key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
        AesLibraryFake.decrypt(key, iv, data)
    }

    fn aes_encrypt(&self, key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
        AesLibraryFake.encrypt(key, iv, data)
    }

    fn aes_decrypt_file(&self, src: String, dst: String, key: String) -> Result<(), String> {
        AesLibraryFake.decrypt_file(src, dst, key)
    }

    fn aes_encrypt_file(&self, src: String, dst: String, key: String) -> Result<(), String> {
        AesLibraryFake.encrypt_file(src, dst, key)
    }

    fn md5(&self, data: Vec<u8>) -> Result<String, String> {
        HashLibraryFake.md5(data)
    }

    fn sha1(&self, data: Vec<u8>) -> Result<String, String> {
        HashLibraryFake.sha1(data)
    }

    fn sha256(&self, data: Vec<u8>) -> Result<String, String> {
        HashLibraryFake.sha256(data)
    }

    fn hash_file(&self, file: String, algo: Option<String>) -> Result<String, String> {
        HashLibraryFake.file(file, algo)
    }

    fn encode_b64(&self, content: String, encode_type: Option<String>) -> Result<String, String> {
        Base64LibraryFake.encode(content, encode_type)
    }

    fn decode_b64(&self, content: String, encode_type: Option<String>) -> Result<String, String> {
        Base64LibraryFake.decode(content, encode_type)
    }

    fn is_json(&self, content: String) -> Result<bool, String> {
        JsonLibraryFake.is_valid(content)
    }

    fn from_json(&self, content: String) -> Result<Value, String> {
        JsonLibraryFake.decode(content)
    }

    fn to_json(&self, content: Value) -> Result<String, String> {
        JsonLibraryFake.encode(content)
    }

    fn encode_utf16le(&self, content: String) -> Result<Vec<u8>, String> {
        match call_bound_method(
            &Value::String(content),
            "encode",
            &[Value::String("utf16le".into())],
        )? {
            Value::Bytes(b) => Ok(b),
            _ => unreachable!(),
        }
    }

    fn decode_utf16le(&self, content: Vec<u8>) -> Result<String, String> {
        match call_bound_method(
            &Value::Bytes(content),
            "decode",
            &[Value::String("utf16le".into())],
        )? {
            Value::String(s) => Ok(s),
            _ => unreachable!(),
        }
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_fake() {
        let crypto = CryptoLibraryFake;
        let data = vec![1, 2, 3];
        let enc = crypto.aes_encrypt(vec![], vec![], data.clone()).unwrap();
        let dec = crypto.aes_decrypt(vec![], vec![], enc).unwrap();
        assert_eq!(data, dec);
    }
}
