use aes::Aes128;
use aes::cipher::{KeyInit, BlockDecrypt, generic_array::GenericArray};
use alloc::string::{String, ToString};
use std::io::Read;

pub fn aes_decrypt_file(src: String, dst: String, key: String) -> Result<(), String> {
    let key_bytes = key.as_bytes();
    if key_bytes.len() != 16 {
        return Err("Key size must be 16 bytes".into());
    }
    let key_arr = GenericArray::from_slice(key_bytes);
    let cipher = Aes128::new(key_arr);

    let mut input = std::fs::File::open(src).map_err(|e| e.to_string())?;
    let len = input.metadata().map_err(|e| e.to_string())?.len();
    if len % 16 != 0 {
        return Err("Input file size is not a multiple of 16 bytes".into());
    }

    let mut output = std::fs::File::create(dst).map_err(|e| e.to_string())?;
    let mut block = GenericArray::from([0u8; 16]);
    let mut next_block = [0u8; 16];

    // Read first block
    if input.read_exact(&mut block).is_err() {
        return Err("Input file is empty or too short".into());
    }

    loop {
        // Try to read next block
        match input.read_exact(&mut next_block) {
            Ok(_) => {
                // Current `block` is not the last one.
                cipher.decrypt_block(&mut block);
                std::io::Write::write_all(&mut output, &block).map_err(|e| e.to_string())?;
                // Move next to current
                block.copy_from_slice(&next_block);
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Current `block` IS the last one.
                cipher.decrypt_block(&mut block);
                // Unpad (PKCS#7) manually
                let last_byte = block[15];
                if last_byte <= 16 && last_byte > 0 {
                    let padding_len = last_byte as usize;
                    let mut valid_padding = true;
                    // Check padding
                    for i in (16 - padding_len)..16 {
                        if block[i] != last_byte {
                            valid_padding = false;
                            break;
                        }
                    }

                    if valid_padding {
                        std::io::Write::write_all(&mut output, &block[..16 - padding_len])
                            .map_err(|e| e.to_string())?;
                    } else {
                        // Invalid padding, write full block? (Mimic aes_decrypt logic which keeps invalid padding)
                        std::io::Write::write_all(&mut output, &block).map_err(|e| e.to_string())?;
                    }
                } else {
                    // Invalid padding length, just write full block
                    std::io::Write::write_all(&mut output, &block).map_err(|e| e.to_string())?;
                }
                break;
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_file_decrypt_invalid_key() {
        assert!(
            aes_decrypt_file("s".into(), "d".into(), "short".into())
                .is_err()
        );
    }

    #[test]
    fn test_aes_file_decrypt_bad_size() -> Result<(), String> {
        let key = "TESTINGPASSWORD!".to_string();

        let mut src_file = NamedTempFile::new().unwrap();
        src_file.write_all(b"123").unwrap(); // Not multiple of 16
        let src_path = src_file.path().to_str().unwrap().to_string();

        let res = aes_decrypt_file(src_path, "dst".into(), key);
        assert!(res.is_err());
        Ok(())
    }
}
