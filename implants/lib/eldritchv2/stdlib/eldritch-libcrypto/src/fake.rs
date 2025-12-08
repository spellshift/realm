use super::CryptoLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;
use super::CryptoLibrary;

#[derive(Default, Debug)]
#[eldritch_library_impl(CryptoLibrary)]
pub struct CryptoLibraryFake;

impl CryptoLibrary for CryptoLibraryFake {
    fn aes_decrypt(&self, _key: Vec<u8>, _iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
        // Mock: just reverse
        let mut d = data;
        d.reverse();
        Ok(d)
    }

    fn aes_encrypt(&self, _key: Vec<u8>, _iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
        // Mock: just reverse
        let mut d = data;
        d.reverse();
        Ok(d)
    }

    fn md5(&self, _data: Vec<u8>) -> Result<String, String> {
        Ok(String::from("d41d8cd98f00b204e9800998ecf8427e")) // Empty md5
    }

    fn sha1(&self, _data: Vec<u8>) -> Result<String, String> {
        Ok(String::from("da39a3ee5e6b4b0d3255bfef95601890afd80709")) // Empty sha1
    }

    fn sha256(&self, _data: Vec<u8>) -> Result<String, String> {
        Ok(String::from(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        )) // Empty sha256
    }

    fn hash_file(&self, _file: String, _algo: String) -> Result<String, String> {
        Err("File hashing not supported in fake/wasm environment".into())
    }

    fn encode_b64(&self, content: String, _encode_type: Option<String>) -> Result<String, String> {
        // Simple mock if needed, or implement using base64 crate if available.
        // For fake/wasm, maybe we can rely on pure rust base64 if available or just return input.
        // But usually we want some encoding.
        // Let's check imports.
        // Just mocking it by prefixing for now or using a simple implementation?
        // Actually, `base64` crate is a dependency of `eldritch-libcrypto`.
        // We can use it if available.
        // But `fake_bindings` usually implies minimal dependencies.
        // Let's just return the content prefixed with "B64:" to prove it was called?
        Ok(format!("B64:{}", content))
    }

    fn decode_b64(&self, content: String, _encode_type: Option<String>) -> Result<String, String> {
        if content.starts_with("B64:") {
            Ok(content[4..].into())
        } else {
            Ok(content)
        }
    }

    fn is_json(&self, _content: String) -> Result<bool, String> {
        Ok(true) // Always pretend valid JSON
    }

    fn from_json(&self, content: String) -> Result<Value, String> {
        Ok(Value::String(content)) // Just return as string for now
    }

    fn to_json(&self, content: Value) -> Result<String, String> {
        Ok(format!("{:?}", content))
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_fake() {
        let crypto = CryptoLibraryFake::default();
        let data = vec![1, 2, 3];
        let enc = crypto.aes_encrypt(vec![], vec![], data.clone()).unwrap();
        let dec = crypto.aes_decrypt(vec![], vec![], enc).unwrap();
        assert_eq!(data, dec);
    }
}
