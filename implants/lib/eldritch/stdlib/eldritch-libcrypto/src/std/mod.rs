use super::CryptoLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::{Value, call_bound_method};
use eldritch_libaes::{AesLibrary, std::StdAesLibrary};
use eldritch_libbase64::{Base64Library, std::StdBase64Library};
use eldritch_libhash::{HashLibrary, std::StdHashLibrary};
use eldritch_libjson::{JsonLibrary, std::StdJsonLibrary};
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(CryptoLibrary)]
pub struct StdCryptoLibrary;

impl CryptoLibrary for StdCryptoLibrary {
    fn aes_encrypt(&self, key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
        StdAesLibrary.encrypt(key, iv, data)
    }

    fn aes_decrypt(&self, key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
        StdAesLibrary.decrypt(key, iv, data)
    }

    fn aes_decrypt_file(&self, src: String, dst: String, key: String) -> Result<(), String> {
        StdAesLibrary.decrypt_file(src, dst, key)
    }

    fn aes_encrypt_file(&self, src: String, dst: String, key: String) -> Result<(), String> {
        StdAesLibrary.encrypt_file(src, dst, key)
    }

    fn md5(&self, data: Vec<u8>) -> Result<String, String> {
        StdHashLibrary.md5(data)
    }

    fn sha1(&self, data: Vec<u8>) -> Result<String, String> {
        StdHashLibrary.sha1(data)
    }

    fn sha256(&self, data: Vec<u8>) -> Result<String, String> {
        StdHashLibrary.sha256(data)
    }

    fn hash_file(&self, file: String, algo: String) -> Result<String, String> {
        StdHashLibrary.file(file, algo)
    }

    fn encode_b64(&self, content: String, encode_type: Option<String>) -> Result<String, String> {
        StdBase64Library.encode(content, encode_type)
    }

    fn decode_b64(&self, content: String, encode_type: Option<String>) -> Result<String, String> {
        StdBase64Library.decode(content, encode_type)
    }

    fn is_json(&self, content: String) -> Result<bool, String> {
        StdJsonLibrary.is_valid(content)
    }

    fn from_json(&self, content: String) -> Result<Value, String> {
        StdJsonLibrary.decode(content)
    }

    fn to_json(&self, content: Value) -> Result<String, String> {
        StdJsonLibrary.encode(content)
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
