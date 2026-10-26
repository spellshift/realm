use aes::Aes128;
use aes::cipher::{
    BlockDecrypt, BlockSizeUser, KeyInit,
    generic_array::{GenericArray, typenum::Unsigned},
};
use alloc::string::String;
use alloc::vec::Vec;

pub fn aes_decrypt(key: Vec<u8>, _iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
    if key.len() != 16 {
        return Err("Key size must be 16 bytes (128 bits)".into());
    }
    let block_size = <Aes128 as BlockSizeUser>::BlockSize::to_usize();
    if !data.len().is_multiple_of(block_size) {
        return Err("Data length must be a multiple of 16 bytes".into());
    }
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let key = GenericArray::from_slice(&key);
    let cipher = Aes128::new(key);

    let mut decrypted_data = data;
    for chunk in decrypted_data.chunks_mut(block_size) {
        let block = GenericArray::from_mut_slice(chunk);
        cipher.decrypt_block(block);
    }

    // Unpadding (PKCS#7)
    if let Some(&pad_len) = decrypted_data.last() {
        let pad_len_usize = pad_len as usize;
        if pad_len_usize == 0 || pad_len_usize > block_size || pad_len_usize > decrypted_data.len()
        {
            return Err("Invalid padding".into());
        }
        for i in 0..pad_len_usize {
            if decrypted_data[decrypted_data.len() - 1 - i] != pad_len {
                return Err("Invalid padding".into());
            }
        }
        decrypted_data.truncate(decrypted_data.len() - pad_len_usize);
        Ok(decrypted_data)
    } else {
        Err("Decryption failed: empty result before unpadding".into())
    }
}

#[cfg(test)]
mod tests {
    use super::super::encrypt_impl::aes_encrypt;
    use super::*;

    #[test]
    fn test_aes_encrypt_decrypt() {
        let key = b"VERY SECRET KEY!".to_vec();
        let iv = vec![0u8; 16]; // Ignored
        let data = b"Hello World!".to_vec();

        let encrypted = aes_encrypt(key.clone(), iv.clone(), data.clone()).unwrap();
        assert_ne!(encrypted, data);
        assert_eq!(encrypted.len() % 16, 0);

        let decrypted = aes_decrypt(key, iv, encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_aes_decrypt_invalid_key() {
        let key = b"SHORT".to_vec();
        let iv = vec![0u8; 16];
        let data = vec![0u8; 16];
        assert!(aes_decrypt(key, iv, data).is_err());
    }

    #[test]
    fn test_aes_decrypt_invalid_data_len() {
        let key = b"VERY SECRET KEY!".to_vec();
        let iv = vec![0u8; 16];
        let data = vec![0u8; 15]; // Not multiple of 16
        assert!(aes_decrypt(key, iv, data).is_err());
    }

    #[test]
    fn test_aes_decrypt_empty() {
        let key = b"VERY SECRET KEY!".to_vec();
        let iv = vec![0u8; 16];
        let data = vec![];
        assert_eq!(aes_decrypt(key, iv, data).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn test_aes_decrypt_bad_padding() {
        let key = b"VERY SECRET KEY!".to_vec();
        let iv = vec![0u8; 16];
        let data = b"Hello World!".to_vec();

        let mut encrypted = aes_encrypt(key.clone(), iv.clone(), data).unwrap();
        // Corrupt the last byte (which affects padding)
        let len = encrypted.len();
        encrypted[len - 1] ^= 0xFF;

        assert!(aes_decrypt(key, iv, encrypted).is_err());
    }
}
