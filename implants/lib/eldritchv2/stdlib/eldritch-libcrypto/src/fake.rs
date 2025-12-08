
use alloc::string::String;
use alloc::vec::Vec;
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
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::CryptoLibraryFake;
    use crate::CryptoLibrary;

    #[test]
    fn test_crypto_fake() {
        let crypto = CryptoLibraryFake::default();
        let data = vec![1, 2, 3];
        let enc = crypto.aes_encrypt(vec![], vec![], data.clone()).unwrap();
        let dec = crypto.aes_decrypt(vec![], vec![], enc).unwrap();
        assert_eq!(data, dec);
    }
}
