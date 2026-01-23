use aes::Aes128;
use aes::cipher::{BlockEncrypt, KeyInit, generic_array::GenericArray};
use alloc::vec::Vec;

pub fn aes_encrypt(key: Vec<u8>, _iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encrypt_invalid_key_length() {
        let key = b"short".to_vec();
        let data = b"data".to_vec();
        let res = aes_encrypt(key, vec![], data);
        assert!(res.is_err());
    }

    #[test]
    fn test_aes_padding_logic() {
        let key = b"TESTINGPASSWORD!".to_vec();
        // Exact block size
        let data = b"1234567890123456".to_vec();

        let encrypted = aes_encrypt(key.clone(), vec![], data.clone()).unwrap();
        // Should produce 2 blocks (32 bytes) because PKCS#7 adds a full block of padding if input is multiple of block size
        assert_eq!(encrypted.len(), 32);
    }

    #[test]
    fn test_aes_vectors() {
        let data = b"Lorem ipsum dolor sit amet".to_vec();
        let key = b"TESTINGPASSWORD!".to_vec();

        let encrypted = aes_encrypt(key.clone(), vec![], data.clone()).unwrap();
        assert!(!encrypted.is_empty());
        assert_eq!(encrypted.len() % 16, 0);
    }
}
