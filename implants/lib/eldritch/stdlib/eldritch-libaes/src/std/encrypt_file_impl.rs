use super::encrypt_impl::aes_encrypt;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use std::fs::File;
use std::io::{Read, Write};

/// Size of the AES block / IV, in bytes.
const IV_LEN: usize = 16;

pub fn aes_encrypt_file(src: String, dst: String, key: String) -> Result<(), String> {
    let key_bytes = key.into_bytes();
    if key_bytes.len() != 16 {
        return Err("Key size must be 16 bytes".into());
    }

    let mut src_file = File::open(src).map_err(|e| e.to_string())?;
    let mut data = Vec::new();
    src_file.read_to_end(&mut data).map_err(|e| e.to_string())?;

    // Generate a fresh random IV for every file so that encrypting the same
    // plaintext twice does not leak information via identical ciphertext.
    let mut iv = [0u8; IV_LEN];
    rand_chacha::ChaCha20Rng::from_entropy().fill_bytes(&mut iv);

    let encrypted = aes_encrypt(key_bytes, iv.to_vec(), data)?;

    // Prepend the IV to the ciphertext so `aes_decrypt_file` can recover it.
    let mut dst_file = File::create(dst).map_err(|e| e.to_string())?;
    dst_file.write_all(&iv).map_err(|e| e.to_string())?;
    dst_file.write_all(&encrypted).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encrypt_file_errors() {
        let key_short = "SHORT".to_string();
        let key_valid = "VERY SECRET KEY!".to_string();

        // Short key
        assert!(aes_encrypt_file("src".to_string(), "dst".to_string(), key_short).is_err());

        // Non-existent source file
        assert!(
            aes_encrypt_file(
                "/non/existent/src".to_string(),
                "dst".to_string(),
                key_valid.clone()
            )
            .is_err()
        );

        // Invalid destination path
        let mut src_file = NamedTempFile::new().unwrap();
        src_file.write_all(b"test").unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();

        assert!(
            aes_encrypt_file(src_path, "/non/existent/dir/dst".to_string(), key_valid).is_err()
        );
    }

    #[test]
    fn test_aes_encrypt_file_random_iv() {
        let key = "VERY SECRET KEY!".to_string();

        let mut src_file = NamedTempFile::new().unwrap();
        src_file.write_all(b"Same plaintext twice.").unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();

        let enc_file_a = NamedTempFile::new().unwrap();
        let enc_path_a = enc_file_a.path().to_str().unwrap().to_string();
        let enc_file_b = NamedTempFile::new().unwrap();
        let enc_path_b = enc_file_b.path().to_str().unwrap().to_string();

        aes_encrypt_file(src_path.clone(), enc_path_a.clone(), key.clone()).unwrap();
        aes_encrypt_file(src_path, enc_path_b.clone(), key).unwrap();

        let mut content_a = Vec::new();
        File::open(&enc_path_a)
            .unwrap()
            .read_to_end(&mut content_a)
            .unwrap();
        let mut content_b = Vec::new();
        File::open(&enc_path_b)
            .unwrap()
            .read_to_end(&mut content_b)
            .unwrap();

        assert_ne!(
            content_a, content_b,
            "encrypting the same file twice should use a fresh random IV each time"
        );
    }
}
