use aes::cipher::BlockSizeUser;
use anyhow::{anyhow, Result};
use std::fs::{rename, File};
use std::io::{Read, Write};
use std::path::Path;
use tempfile::NamedTempFile;

use aes::cipher::{generic_array::GenericArray, BlockDecrypt, KeyInit};
use aes::Aes128;

pub fn decrypt_file(src: String, dst: String, key: String) -> Result<()> {
    if !Path::new(&dst).exists() {
        File::create(dst.clone())?;
    }
    let key_bytes = key.as_bytes();
    if key_bytes.len() != 16 {
        return Err(anyhow!("Key size must be 16 bytes (characters)"));
    }
    let key_bytes: [u8; 16] = key_bytes[..16].try_into()?;
    let key = GenericArray::from(key_bytes);
    let mut block = GenericArray::from([0u8; 16]);
    let cipher = Aes128::new(&key);
    let mut src_file = File::open(src.clone())?;
    let mut src_len = src_file.metadata()?.len();
    if src_len % Aes128::block_size() as u64 != 0 {
        return Err(anyhow!("File size must be a multiple of 16 bytes"));
    }
    let mut out_file = NamedTempFile::new()?;
    while let Ok(_n) = src_file.read(&mut block[..]) {
        if src_len == 0 {
            break;
        }
        cipher.decrypt_block(&mut block);
        src_len -= 16;
        if src_len == 0 {
            let last_byte = block[15];
            if last_byte < 16 && last_byte > 0 {
                let suspected_padding = &block[(16 - last_byte) as usize..=15];
                let mut invalid = false;
                for byte in suspected_padding {
                    if byte != &last_byte {
                        invalid = true;
                        break;
                    }
                }
                if !invalid {
                    match out_file.write_all(&block[..(16 - last_byte) as usize]) {
                        Ok(_) => {}
                        Err(_err) => {
                            #[cfg(debug_assertions)]
                            log::error!("failed to decrypt file: {_err}");
                        }
                    };
                    continue;
                }
            }
        }
        match out_file.write_all(&block) {
            Ok(_) => {}
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!("failed to decrypt file: {_err}");
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

    use super::decrypt_file;

    use hex_literal::hex;
    use sha1::{Digest, Sha1};

    #[test]
    fn test_decrypt() -> Result<()> {
        let lorem_encrypted = hex!("9c0a 9d0b bf63 8f19 e8bd f28f f742 9513 fdbb c517 4c9a 8e29 473d a80e 4c38 f052 386b 4bf4 b432 e270 3090 f4dd 4cf6 dfb5 1802 9c49 e5e7 32e3 ec6e fe2a 3ba9 1bba 78bd 6752 08ea 520d 5ee4 a116 b24d 889e 0d5a 6da2 6baf 5d55 6122 28f6 1741 4035 eeed 2fab 2597 5de6 80f6 cff4 5308 06c6 c2e1 30ba 88e6 0654 80f5 af9a 03bf 0af6 2940 7a2b e1a4 ab09 8551 98e5 f455 235b 6094 18aa a388 974f 9580 39fa eed1 20f8 2754 5666 cb25 4b24 9dd8 7cf7 9c8a 4161 f4ef dc44 3d2a 1b9a ab66 5b4d 8bfd 82aa 70a9 cbba ede3 af9a 2e78 04d5 ecb4 5387 6594 8662 dd7a 90ad b03d 9a57 c7a4 d17d 4373 29b1 d073 92b2 9ae5 6da9 af4d 9ba7 edd8 2e82 1846 1355 9cc0 9707 c946 b805 ca7f e9bb 6f0e 64fe bfda de74 f61d 9138 7b8b 48f0 4d48 78f0 1c35 6970 f7b7 22ed da6e dddb 0d1f 2e21 b952 a592 bcde 0823 7329 372a 8c0c e824 58d9 ad36 2282 dfa6 48b1 0e50 fc77 cbd9 3b02 f80c ca2d cf46 194d d1b7 0f36 f7e6 6abb fbd1 13e2 083a ab0f 2835 b7bd d820 052e f7cf f6cb c30f c22e be5b 7372 a8c1 f4e0 8a2a 1602 d43e de29 3282 ecd6 1aee 4bbb a2b8 9bbf 7693 5e21 3c02 4bc0 910d e01c 1228 d219 b7e1 895e 303e 0c85 c375 7de3 bd3f 4b5f da33 82bf 34db 298b 06d2 40a6 81c0 a70f 4e1f");
        let tmp_dir = TempDir::new()?;
        let test_path = tmp_dir.path().join("test.txt");
        let test_dec_path = tmp_dir.path().join("test.txt.dec");
        {
            let mut tmp_file = File::create(test_path.clone())?;
            match tmp_file.write_all(&lorem_encrypted) {
                Ok(_) => {}
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to decrypt file: {_err}");
                }
            };
        }
        decrypt_file(
            test_path.to_str().unwrap().to_owned(),
            test_dec_path.to_str().unwrap().to_owned(),
            "TESTINGPASSWORD!".to_string(),
        )?;

        let mut hasher = Sha1::new();
        let mut dec_f = File::open(test_dec_path)?;
        let mut dec_f_content = Vec::new();
        dec_f.read_to_end(&mut dec_f_content)?;
        hasher.update(dec_f_content);
        let result = hasher.finalize();

        assert_eq!(result[..], hex!("4518012e1b365e504001dbc94120624f15b8bbd5"));
        Ok(())
    }

    #[test]
    fn test_decrypt_bad_password() -> Result<()> {
        let tmp_dir = TempDir::new()?;
        let test_path = tmp_dir.path().join("test.txt");
        assert!(decrypt_file(
            test_path.to_str().unwrap().to_owned(),
            test_path.to_str().unwrap().to_owned(),
            "TESTINGPASSWORD!!".to_string()
        )
        .is_err());
        assert!(decrypt_file(
            test_path.to_str().unwrap().to_owned(),
            test_path.to_str().unwrap().to_owned(),
            "TESTINGPASSWORD".to_string()
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn test_decrypt_no_file() -> Result<()> {
        let tmp_dir = TempDir::new()?;
        let test_path = tmp_dir.path().join("test.txt");
        assert!(decrypt_file(
            "/I/Dont/Exist".to_string(),
            test_path.to_str().unwrap().to_owned(),
            "TESTINGPASSWORD!".to_string()
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn test_decrypt_bad_size() -> Result<()> {
        let tmp_dir = TempDir::new()?;
        let test_path = tmp_dir.path().join("test.txt");
        {
            let mut tmp_file = File::create(test_path.clone())?;
            match tmp_file.write_all(&[0u8; 15]) {
                Ok(_) => {}
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to decrypt file: {_err}");
                }
            };
        }
        assert!(decrypt_file(
            test_path.to_str().unwrap().to_owned(),
            test_path.to_str().unwrap().to_owned(),
            "TESTINGPASSWORD!".to_string()
        )
        .is_err());
        Ok(())
    }
}
