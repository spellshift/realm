use super::AesLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(AesLibrary)]
pub struct AesLibraryFake;

impl AesLibrary for AesLibraryFake {
    fn decrypt(&self, _key: Vec<u8>, _iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
        let mut d = data;
        d.reverse();
        Ok(d)
    }

    fn decrypt_file(&self, _src: String, _dst: String, _key: String) -> Result<(), String> {
        Err("File decryption not supported in fake/wasm environment".into())
    }

    fn encrypt(&self, _key: Vec<u8>, _iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
        let mut d = data;
        d.reverse();
        Ok(d)
    }

    fn encrypt_file(&self, _src: String, _dst: String, _key: String) -> Result<(), String> {
        Err("File encryption not supported in fake/wasm environment".into())
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_aes_fake() {
        let aes = AesLibraryFake;
        let data = vec![1, 2, 3];
        let enc = aes.encrypt(vec![], vec![], data.clone()).unwrap();
        let dec = aes.decrypt(vec![], vec![], enc).unwrap();
        assert_eq!(data, dec);
    }
}
