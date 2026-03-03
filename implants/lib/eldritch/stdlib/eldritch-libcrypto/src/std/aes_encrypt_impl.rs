use aes::Aes128;
use aes::cipher::{BlockEncrypt, KeyInit, generic_array::GenericArray};
use alloc::vec::Vec;
use bytes::Bytes;

pub fn aes_encrypt(key: Bytes, _iv: Bytes, data: Bytes) -> Result<Bytes, String> {
    if key.len() != 16 {
        return Err("Key size must be 16 bytes (characters)".into());
    }
    let key_bytes: [u8; 16] = key.as_ref().try_into().map_err(|_| "Key size mismatch")?;
    let key_arr = GenericArray::from(key_bytes);

    // Pad data (PKCS#7)
    let mut padded_data = data.to_vec();
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

    Ok(Bytes::from(output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encrypt_invalid_key_length() {
        let key = Bytes::from_static(b"short");
        let data = Bytes::from_static(b"data");
        let res = aes_encrypt(key, Bytes::new(), data);
        assert!(res.is_err());
    }

    #[test]
    fn test_aes_padding_logic() {
        let key = Bytes::from_static(b"TESTINGPASSWORD!");
        // Exact block size
        let data = Bytes::from_static(b"1234567890123456");

        let encrypted = aes_encrypt(key.clone(), Bytes::new(), data.clone()).unwrap();
        // Should produce 2 blocks (32 bytes) because PKCS#7 adds a full block of padding if input is multiple of block size
        assert_eq!(encrypted.len(), 32);
    }

    #[test]
    fn test_aes_vectors() {
        let data = Bytes::from_static(b"Lorem ipsum dolor sit amet");
        let key = Bytes::from_static(b"TESTINGPASSWORD!");

        let encrypted = aes_encrypt(key.clone(), Bytes::new(), data.clone()).unwrap();
        assert!(!encrypted.is_empty());
        assert_eq!(encrypted.len() % 16, 0);
    }
}
