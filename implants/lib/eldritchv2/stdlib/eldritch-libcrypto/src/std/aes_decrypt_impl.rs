use aes::Aes128;
use aes::cipher::{BlockDecrypt, KeyInit, generic_array::GenericArray};
use alloc::vec::Vec;

pub fn aes_decrypt(key: Vec<u8>, _iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
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
    if let Some(&last_byte) = output.last()
        && last_byte <= 16
        && last_byte > 0
    {
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

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::aes_encrypt_impl::aes_encrypt;

    #[test]
    fn test_aes_roundtrip() {
        let key = b"TESTINGPASSWORD!".to_vec();
        let iv = vec![0u8; 16]; // Ignored
        let data = b"Hello World!".to_vec();

        let encrypted = aes_encrypt(key.clone(), iv.clone(), data.clone())
            .expect("encrypt failed");
        assert_ne!(encrypted, data);
        assert_eq!(encrypted.len() % 16, 0);

        let decrypted = aes_decrypt(key.clone(), iv.clone(), encrypted)
            .expect("decrypt failed");
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_aes_decrypt_invalid_key_length() {
        let key = b"short".to_vec();
        let data = b"data".to_vec();
        let res = aes_decrypt(key, vec![], data);
        assert!(res.is_err());
    }

    #[test]
    fn test_aes_decrypt_invalid_data_length() {
        let key = b"TESTINGPASSWORD!".to_vec();
        let data = b"not_multiple_16".to_vec();
        let res = aes_decrypt(key, vec![], data);
        assert!(res.is_err());
    }

    #[test]
    fn test_aes_decrypt_invalid_padding() {
        let key = b"TESTINGPASSWORD!".to_vec();
        let data = b"data".to_vec();
        let mut encrypted = aes_encrypt(key.clone(), vec![], data).unwrap();

        // Modify last byte to make padding invalid
        if let Some(last) = encrypted.last_mut() {
            *last ^= 0xFF; // Flip bits
        }

        let decrypted = aes_decrypt(key, vec![], encrypted).unwrap();
        // Since padding is invalid, it won't be stripped.
        // Original data length was 4. Padded to 16.
        // Encrypted length is 16.
        // Decrypted length should remain 16 because padding check failed.
        assert_eq!(decrypted.len(), 16);
    }
}
