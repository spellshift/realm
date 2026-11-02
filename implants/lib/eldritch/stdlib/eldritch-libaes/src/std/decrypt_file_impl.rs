use super::decrypt_impl::aes_decrypt;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::fs::File;
use std::io::{Read, Write};

pub fn aes_decrypt_file(src: String, dst: String, key: String) -> Result<(), String> {
    let key_bytes = key.into_bytes();
    if key_bytes.len() != 16 {
        return Err("Key size must be 16 bytes".into());
    }

    let mut src_file = File::open(src).map_err(|e| e.to_string())?;
    let mut data = Vec::new();
    src_file.read_to_end(&mut data).map_err(|e| e.to_string())?;

    let decrypted = aes_decrypt(key_bytes, Vec::new(), data)?;

    let mut dst_file = File::create(dst).map_err(|e| e.to_string())?;
    dst_file.write_all(&decrypted).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::encrypt_file_impl::aes_encrypt_file;
    use super::*;
    use std::io::{Read, Write};
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_file_encrypt_decrypt() {
        let mut src_file = NamedTempFile::new().unwrap();
        let data = b"Secret file content that needs encryption.";
        src_file.write_all(data).unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();

        let enc_file = NamedTempFile::new().unwrap();
        let enc_path = enc_file.path().to_str().unwrap().to_string();

        let dec_file = NamedTempFile::new().unwrap();
        let dec_path = dec_file.path().to_str().unwrap().to_string();

        let key = "VERY SECRET KEY!".to_string();

        aes_encrypt_file(src_path, enc_path.clone(), key.clone()).unwrap();

        let mut enc_content = Vec::new();
        File::open(&enc_path)
            .unwrap()
            .read_to_end(&mut enc_content)
            .unwrap();
        assert_ne!(enc_content, data);

        aes_decrypt_file(enc_path, dec_path.clone(), key).unwrap();

        let mut dec_content = Vec::new();
        File::open(&dec_path)
            .unwrap()
            .read_to_end(&mut dec_content)
            .unwrap();
        assert_eq!(dec_content, data);
    }

    #[test]
    fn test_aes_decrypt_file_errors() {
        let key_short = "SHORT".to_string();
        let key_valid = "VERY SECRET KEY!".to_string();

        // Short key
        assert!(aes_decrypt_file("src".to_string(), "dst".to_string(), key_short).is_err());

        // Non-existent source file
        assert!(
            aes_decrypt_file(
                "/non/existent/src".to_string(),
                "dst".to_string(),
                key_valid.clone()
            )
            .is_err()
        );

        // Invalid destination path
        let mut src_file = NamedTempFile::new().unwrap();
        let data = b"Some data";
        src_file.write_all(data).unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();

        let enc_file = NamedTempFile::new().unwrap();
        let enc_path = enc_file.path().to_str().unwrap().to_string();
        aes_encrypt_file(src_path, enc_path.clone(), key_valid.clone()).unwrap();

        assert!(
            aes_decrypt_file(enc_path, "/non/existent/dir/dst".to_string(), key_valid).is_err()
        );
    }
}
