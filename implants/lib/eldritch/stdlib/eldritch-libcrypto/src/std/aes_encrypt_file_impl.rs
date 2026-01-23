use aes::Aes128;
use aes::cipher::{BlockEncrypt, KeyInit, generic_array::GenericArray};
use alloc::string::{String, ToString};
use std::io::{Read, Write};

pub fn aes_encrypt_file(src: String, dst: String, key: String) -> Result<(), String> {
    let key_bytes = key.as_bytes();
    if key_bytes.len() != 16 {
        return Err("Key size must be 16 bytes".into());
    }
    let key_arr = GenericArray::from_slice(key_bytes);
    let cipher = Aes128::new(key_arr);

    let mut input = std::fs::File::open(src).map_err(|e| e.to_string())?;
    let mut output = std::fs::File::create(dst).map_err(|e| e.to_string())?;

    let mut temp_buf = [0u8; 16];

    loop {
        // Read loop to fill 16 bytes or hit EOF
        let mut chunk_len = 0;
        while chunk_len < 16 {
            let n = input
                .read(&mut temp_buf[chunk_len..])
                .map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            chunk_len += n;
        }

        if chunk_len == 16 {
            // We have a full block. Encrypt and write.
            let mut block = GenericArray::clone_from_slice(&temp_buf);
            cipher.encrypt_block(&mut block);
            output.write_all(&block).map_err(|e| e.to_string())?;
        } else {
            // Last block (partial or empty). Pad.
            let padding_byte = (16 - chunk_len) as u8;
            temp_buf[chunk_len..16].fill(padding_byte);
            let mut block = GenericArray::clone_from_slice(&temp_buf);
            cipher.encrypt_block(&mut block);
            output.write_all(&block).map_err(|e| e.to_string())?;
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::aes_decrypt_file_impl::aes_decrypt_file;
    use super::*;
    use std::io::{Read, Write};
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_file_roundtrip() -> Result<(), String> {
        let key = "TESTINGPASSWORD!".to_string();
        let data = b"Hello World! This is a test file for AES encryption.".to_vec();

        // Write src
        let mut src_file = NamedTempFile::new().map_err(|e| e.to_string())?;
        src_file.write_all(&data).map_err(|e| e.to_string())?;
        let src_path = src_file.path().to_str().unwrap().to_string();

        let dst_file = NamedTempFile::new().map_err(|e| e.to_string())?;
        let dst_path = dst_file.path().to_str().unwrap().to_string();

        let dec_file = NamedTempFile::new().map_err(|e| e.to_string())?;
        let dec_path = dec_file.path().to_str().unwrap().to_string();

        // Encrypt
        aes_encrypt_file(src_path.clone(), dst_path.clone(), key.clone())?;

        // Verify dst size
        let dst_len = std::fs::metadata(&dst_path).unwrap().len();
        assert_eq!(dst_len % 16, 0);

        // Decrypt
        aes_decrypt_file(dst_path.clone(), dec_path.clone(), key.clone())?;

        // Verify content
        let mut result_data = Vec::new();
        std::fs::File::open(&dec_path)
            .unwrap()
            .read_to_end(&mut result_data)
            .unwrap();
        assert_eq!(result_data, data);

        Ok(())
    }

    #[test]
    fn test_aes_file_encrypt_empty() -> Result<(), String> {
        let key = "TESTINGPASSWORD!".to_string();

        let src_file = NamedTempFile::new().unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();
        let dst_file = NamedTempFile::new().unwrap();
        let dst_path = dst_file.path().to_str().unwrap().to_string();

        aes_encrypt_file(src_path, dst_path.clone(), key).unwrap();

        // Empty file padded to 1 block
        let dst_len = std::fs::metadata(&dst_path).unwrap().len();
        assert_eq!(dst_len, 16);

        Ok(())
    }
}
