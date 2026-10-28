use super::HashLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;

#[derive(Default, Debug)]
#[eldritch_library_impl(HashLibrary)]
pub struct HashLibraryFake;

impl HashLibrary for HashLibraryFake {
    fn file(&self, _file: String, algo: Option<String>) -> Result<String, String> {
        let algo = algo.unwrap_or_else(|| "SHA256".into());
        match algo.to_uppercase().as_str() {
            "MD5" => Ok("5d41402abc4b2a76b9719d911017c592".into()),
            "SHA1" => Ok("aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d".into()),
            "SHA256" => {
                Ok("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824".into())
            }
            "SHA512" => Ok(
                "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043"
                    .into(),
            ),
            _ => Err("Unknown algorithm".into()),
        }
    }

    fn md5(&self, _data: Vec<u8>) -> Result<String, String> {
        Ok("5d41402abc4b2a76b9719d911017c592".into())
    }

    fn sha1(&self, _data: Vec<u8>) -> Result<String, String> {
        Ok("aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d".into())
    }

    fn sha256(&self, _data: Vec<u8>) -> Result<String, String> {
        Ok("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824".into())
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_fake_hash() {
        let hash = HashLibraryFake;
        assert_eq!(hash.md5(vec![]).unwrap().len(), 32);
        assert_eq!(hash.sha1(vec![]).unwrap().len(), 40);
        assert_eq!(hash.sha256(vec![]).unwrap().len(), 64);
        assert_eq!(hash.file("test".to_string(), None).unwrap().len(), 64);
        assert_eq!(
            hash.file("test".to_string(), Some("MD5".to_string()))
                .unwrap()
                .len(),
            32
        );
        assert!(
            hash.file("test".to_string(), Some("INVALID".to_string()))
                .is_err()
        );
    }
}
