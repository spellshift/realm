use super::CryptoLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Bytes;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(CryptoLibrary)]
pub struct CryptoLibraryFake;

impl CryptoLibrary for CryptoLibraryFake {
    fn aes_decrypt(&self, _key: Bytes, _iv: Bytes, data: Bytes) -> Result<Bytes, String> {
        // Mock: just reverse
        let mut d = data.to_vec();
        d.reverse();
        Ok(Bytes::from(d))
    }

    fn aes_encrypt(&self, _key: Bytes, _iv: Bytes, data: Bytes) -> Result<Bytes, String> {
        // Mock: just reverse
        let mut d = data.to_vec();
        d.reverse();
        Ok(Bytes::from(d))
    }

    fn aes_decrypt_file(&self, _src: String, _dst: String, _key: String) -> Result<(), String> {
        Err("File decryption not supported in fake/wasm environment".into())
    }

    fn aes_encrypt_file(&self, _src: String, _dst: String, _key: String) -> Result<(), String> {
        Err("File encryption not supported in fake/wasm environment".into())
    }

    fn md5(&self, _data: Bytes) -> Result<String, String> {
        Ok(String::from("d41d8cd98f00b204e9800998ecf8427e")) // Empty md5
    }

    fn sha1(&self, _data: Bytes) -> Result<String, String> {
        Ok(String::from("da39a3ee5e6b4b0d3255bfef95601890afd80709")) // Empty sha1
    }

    fn sha256(&self, _data: Bytes) -> Result<String, String> {
        Ok(String::from(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        )) // Empty sha256
    }

    fn hash_file(&self, _file: String, _algo: String) -> Result<String, String> {
        Err("File hashing not supported in fake/wasm environment".into())
    }

    fn encode_b64(&self, content: String, _encode_type: Option<String>) -> Result<String, String> {
        Ok(format!("B64:{}", content))
    }

    fn decode_b64(&self, content: String, _encode_type: Option<String>) -> Result<String, String> {
        if let Some(stripped) = content.strip_prefix("B64:") {
            Ok(stripped.into())
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
        let crypto = CryptoLibraryFake;
        let data = Bytes::from(&[1, 2, 3]);
        let enc = crypto
            .aes_encrypt(Bytes::new(), Bytes::new(), data.clone())
            .unwrap();
        let dec = crypto
            .aes_decrypt(Bytes::new(), Bytes::new(), enc)
            .unwrap();
        assert_eq!(data.as_ref(), dec.as_ref());
    }
}
