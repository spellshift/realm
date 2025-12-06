use super::CryptoLibrary;
use aes::cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes128;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;
use md5::Context as Md5Context;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use std::io::Read;

#[derive(Default, Debug)]
#[eldritch_library_impl(CryptoLibrary)]
pub struct StdCryptoLibrary;

impl CryptoLibrary for StdCryptoLibrary {
    fn aes_encrypt(&self, key: Vec<u8>, _iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
        if key.len() != 16 {
            return Err("Key size must be 16 bytes (characters)".into());
        }
        let key_bytes: [u8; 16] = key.as_slice().try_into().map_err(|_| "Key size mismatch")?;
        let key_arr = GenericArray::from(key_bytes);

        // Pad data (PKCS#7)
        let mut padded_data = data.clone();
        let padding_needed = 16 - (padded_data.len() % 16);
        for _ in 0..padding_needed {
            padded_data.push(padding_needed as u8);
        }

        let cipher = Aes128::new(&key_arr);
        let mut block = GenericArray::from([0u8; 16]);
        let mut output = Vec::with_capacity(padded_data.len());

        for chunk in padded_data.chunks(16) {
            block.copy_from_slice(chunk);
            cipher.encrypt_block(&mut block);
            output.extend_from_slice(block.as_slice());
        }

        Ok(output)
    }

    fn aes_decrypt(&self, key: Vec<u8>, _iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
        if key.len() != 16 {
            return Err("Key size must be 16 bytes (characters)".into());
        }
        if !data.len().is_multiple_of(16) {
            return Err("Data size must be a multiple of 16 bytes".into());
        }

        let key_bytes: [u8; 16] = key.as_slice().try_into().map_err(|_| "Key size mismatch")?;
        let key_arr = GenericArray::from(key_bytes);

        let cipher = Aes128::new(&key_arr);
        let mut block = GenericArray::from([0u8; 16]);
        let mut output = Vec::with_capacity(data.len());

        // Decrypt all blocks
        for chunk in data.chunks(16) {
            block.copy_from_slice(chunk);
            cipher.decrypt_block(&mut block);
            output.extend_from_slice(block.as_slice());
        }

        // Unpad (PKCS#7) manually to match v1 logic
        if let Some(&last_byte) = output.last() {
            if last_byte <= 16 && last_byte > 0 {
                let len = output.len();
                let start_padding = len - (last_byte as usize);
                if start_padding < len {
                    // Check bound
                    let suspected_padding = &output[start_padding..];
                    let mut valid_padding = true;
                    for &byte in suspected_padding {
                        if byte != last_byte {
                            valid_padding = false;
                            break;
                        }
                    }

                    if valid_padding {
                        output.truncate(start_padding);
                    }
                }
            }
        }

        Ok(output)
    }

    fn md5(&self, data: Vec<u8>) -> Result<String, String> {
        Ok(format!("{:02x}", md5::compute(data)))
    }

    fn sha1(&self, data: Vec<u8>) -> Result<String, String> {
        let mut hasher = Sha1::new();
        hasher.update(&data);
        Ok(format!("{:02x}", hasher.finalize()))
    }

    fn sha256(&self, data: Vec<u8>) -> Result<String, String> {
        let mut hasher = Sha256::new();
        hasher.update(&data);
        Ok(format!("{:02x}", hasher.finalize()))
    }

    fn hash_file(&self, file: String, algo: String) -> Result<String, String> {
        let file = std::fs::File::open(file).map_err(|e| e.to_string())?;
        let mut reader = std::io::BufReader::new(file);
        let mut buffer = [0; 8192];

        // Helper closure to process the file in chunks
        let mut process = |feed: &mut dyn FnMut(&[u8])| -> Result<(), String> {
            loop {
                let count = reader.read(&mut buffer).map_err(|e| e.to_string())?;
                if count == 0 {
                    break;
                }
                feed(&buffer[..count]);
            }
            Ok(())
        };

        match algo.to_lowercase().as_str() {
            "md5" => {
                let mut hasher = Md5Context::new();
                process(&mut |chunk| hasher.consume(chunk))?;
                Ok(format!("{:02x}", hasher.compute()))
            }
            "sha1" => {
                let mut hasher = Sha1::new();
                process(&mut |chunk| hasher.update(chunk))?;
                Ok(format!("{:02x}", hasher.finalize()))
            }
            "sha256" => {
                let mut hasher = Sha256::new();
                process(&mut |chunk| hasher.update(chunk))?;
                Ok(format!("{:02x}", hasher.finalize()))
            }
            "sha512" => {
                let mut hasher = Sha512::new();
                process(&mut |chunk| hasher.update(chunk))?;
                Ok(format!("{:02x}", hasher.finalize()))
            }
            _ => Err(format!("Unknown algorithm: {}", algo)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_roundtrip() {
        let lib = StdCryptoLibrary;
        let key = b"TESTINGPASSWORD!".to_vec();
        let iv = vec![0u8; 16]; // Ignored
        let data = b"Hello World!".to_vec();

        let encrypted = lib
            .aes_encrypt(key.clone(), iv.clone(), data.clone())
            .expect("encrypt failed");
        assert_ne!(encrypted, data);
        assert_eq!(encrypted.len() % 16, 0);

        let decrypted = lib
            .aes_decrypt(key.clone(), iv.clone(), encrypted)
            .expect("decrypt failed");
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_aes_padding_logic() {
        let lib = StdCryptoLibrary;
        let key = b"TESTINGPASSWORD!".to_vec();
        // Exact block size
        let data = b"1234567890123456".to_vec();

        let encrypted = lib.aes_encrypt(key.clone(), vec![], data.clone()).unwrap();
        // Should produce 2 blocks (32 bytes) because PKCS#7 adds a full block of padding if input is multiple of block size
        assert_eq!(encrypted.len(), 32);

        let decrypted = lib.aes_decrypt(key.clone(), vec![], encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_aes_vectors() {
        // We can test against the v1 implementation's implicit logic
        // "Lorem ipsum..." (truncated for brevity)
        let data = b"Lorem ipsum dolor sit amet".to_vec();
        let key = b"TESTINGPASSWORD!".to_vec();
        let lib = StdCryptoLibrary;

        let encrypted = lib.aes_encrypt(key.clone(), vec![], data.clone()).unwrap();
        let decrypted = lib.aes_decrypt(key.clone(), vec![], encrypted).unwrap();

        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_md5() {
        let lib = StdCryptoLibrary;
        let data = b"hello world".to_vec();
        let hash = lib.md5(data).unwrap();
        assert_eq!(hash, "5eb63bbbe01eeed093cb22bb8f5acdc3");
    }

    #[test]
    fn test_sha1() {
        let lib = StdCryptoLibrary;
        let data = b"hello world".to_vec();
        let hash = lib.sha1(data).unwrap();
        assert_eq!(hash, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
    }

    #[test]
    fn test_sha256() {
        let lib = StdCryptoLibrary;
        let data = b"hello world".to_vec();
        let hash = lib.sha256(data).unwrap();
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_hash_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let lib = StdCryptoLibrary;
        let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
        let lorem_hash_md5 = "db89bb5ceab87f9c0fcc2ab36c189c2c";
        let lorem_hash_sha1 = "cd36b370758a259b34845084a6cc38473cb95e27";
        let lorem_hash_sha256 = "2d8c2f6d978ca21712b5f6de36c9d31fa8e96a4fa5d8ff8b0188dfb9e7c171bb";
        let lorem_hash_sha512 = "8ba760cac29cb2b2ce66858ead169174057aa1298ccd581514e6db6dee3285280ee6e3a54c9319071dc8165ff061d77783100d449c937ff1fb4cd1bb516a69b9";

        let mut tmp_file = NamedTempFile::new().expect("failed to create temp file");
        write!(tmp_file, "{}", lorem).expect("failed to write to temp file");
        let path = String::from(tmp_file.path().to_str().unwrap());

        assert_eq!(
            lib.hash_file(path.clone(), "md5".to_string()).unwrap(),
            lorem_hash_md5
        );
        assert_eq!(
            lib.hash_file(path.clone(), "sha1".to_string()).unwrap(),
            lorem_hash_sha1
        );
        assert_eq!(
            lib.hash_file(path.clone(), "sha256".to_string()).unwrap(),
            lorem_hash_sha256
        );
        assert_eq!(
            lib.hash_file(path.clone(), "sha512".to_string()).unwrap(),
            lorem_hash_sha512
        );
    }

    #[test]
    fn test_hash_file_invalid_algo() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let lib = StdCryptoLibrary;
        let mut tmp_file = NamedTempFile::new().expect("failed to create temp file");
        write!(tmp_file, "test").expect("failed to write to temp file");
        let path = String::from(tmp_file.path().to_str().unwrap());

        assert!(lib.hash_file(path, "invalid".to_string()).is_err());
    }

    #[test]
    fn test_hash_file_not_found() {
        let lib = StdCryptoLibrary;
        assert!(lib
            .hash_file("/non/existent/file".to_string(), "md5".to_string())
            .is_err());
    }
}
