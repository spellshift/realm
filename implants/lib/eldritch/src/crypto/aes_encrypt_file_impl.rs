use aes::cipher::{generic_array::GenericArray, BlockEncrypt, KeyInit};
use aes::Aes128;
use anyhow::{anyhow, Result};
use std::fs::{rename, File};
use std::io::{Read, Write};
use std::path::Path;
use tempfile::NamedTempFile;

pub fn encrypt_file(src: String, dst: String, key: String) -> Result<()> {
    if !Path::new(&dst).exists() {
        File::create(dst.clone())?;
    }
    let key_bytes = key.as_bytes();
    if key_bytes.len() != 16 {
        return Err(anyhow!("Key size must be 16 bytes (characters)"));
    }
    let key_bytes: [u8; 16] = key_bytes[..16].try_into()?;
    let key = GenericArray::from(key_bytes);

    let mut block = GenericArray::from([0; 16]);
    let cipher = Aes128::new(&key);
    let mut src_file = File::open(src.clone())?;
    let mut out_file = NamedTempFile::new()?;
    while let Ok(n) = src_file.read(&mut block[..]) {
        if n == 0 {
            break;
        }
        if n != 16 {
            let mut short_buffer = Vec::with_capacity(n);
            for i in 0..n {
                short_buffer.push(block[i]);
            }
            for _ in 0..(16 - n) {
                short_buffer.push((16 - n).try_into()?);
            }
            block = GenericArray::from_iter(short_buffer);
        }
        cipher.encrypt_block(&mut block);
        match out_file.write_all(&block) {
            Ok(_) => {}
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!("failed to encrypt file: {_err}");
            }
        };
        block = GenericArray::from([0u8; 16]);
    }
    drop(src_file);
    rename(out_file.path(), dst)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{Read, Write},
    };

    use anyhow::Result;
    use tempfile::TempDir;

    use super::encrypt_file;

    use hex_literal::hex;
    use sha1::{Digest, Sha1};

    #[test]
    fn test_encrypt() -> Result<()> {
        let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.\n";
        let tmp_dir = TempDir::new()?;
        let test_path = tmp_dir.path().join("test.txt");
        let test_enc_path = tmp_dir.path().join("test.txt.enc");
        {
            let mut tmp_file = File::create(test_path.clone())?;
            write!(tmp_file, "{}", lorem)?;
        }
        encrypt_file(
            test_path.to_str().unwrap().to_owned(),
            test_enc_path.to_str().unwrap().to_owned(),
            "TESTINGPASSWORD!".to_string(),
        )?;

        let mut hasher = Sha1::new();
        let mut enc_f = File::open(test_enc_path)?;
        let mut enc_f_content = Vec::new();
        enc_f.read_to_end(&mut enc_f_content)?;
        hasher.update(enc_f_content);
        let result = hasher.finalize();

        assert_eq!(result[..], hex!("df8a71c3a05157dc6dfc3ea3a82811e1a45e6b9c"));
        Ok(())
    }

    #[test]
    fn test_encrypt_bad_password() -> Result<()> {
        let tmp_dir = TempDir::new()?;
        let test_path = tmp_dir.path().join("test.txt");
        assert!(encrypt_file(
            test_path.to_str().unwrap().to_owned(),
            test_path.to_str().unwrap().to_owned(),
            "TESTINGPASSWORD!!".to_string()
        )
        .is_err());
        assert!(encrypt_file(
            test_path.to_str().unwrap().to_owned(),
            test_path.to_str().unwrap().to_owned(),
            "TESTINGPASSWORD".to_string()
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn test_encrypt_no_file() -> Result<()> {
        let tmp_dir = TempDir::new()?;
        let test_path = tmp_dir.path().join("test.txt");
        assert!(encrypt_file(
            "/I/Dont/Exist".to_string(),
            test_path.to_str().unwrap().to_owned(),
            "TESTINGPASSWORD!".to_string()
        )
        .is_err());
        Ok(())
    }
}
