use super::encrypt_impl::aes_encrypt;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::fs::File;
use std::io::{Read, Write};

pub fn aes_encrypt_file(src: String, dst: String, key: String) -> Result<(), String> {
    let key_bytes = key.into_bytes();
    if key_bytes.len() != 16 {
        return Err("Key size must be 16 bytes".into());
    }

    let mut src_file = File::open(src).map_err(|e| e.to_string())?;
    let mut data = Vec::new();
    src_file.read_to_end(&mut data).map_err(|e| e.to_string())?;

    let encrypted = aes_encrypt(key_bytes, Vec::new(), data)?;

    let mut dst_file = File::create(dst).map_err(|e| e.to_string())?;
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
}
