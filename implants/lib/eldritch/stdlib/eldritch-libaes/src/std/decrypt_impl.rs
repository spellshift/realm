use aes::Aes128;
use aes::cipher::{
    BlockDecrypt, BlockSizeUser, KeyInit,
    generic_array::{GenericArray, typenum::Unsigned},
};
use alloc::string::String;
use alloc::vec::Vec;

/// Decrypts `data` with AES-128 in CBC mode, removing PKCS#7 padding.
///
/// `key` must be exactly 16 bytes and `iv` must be exactly 16 bytes (the AES
/// block size). The IV is mixed into every block via CBC chaining; it is not
/// ignored.
pub fn aes_decrypt(key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
    if key.len() != 16 {
        return Err("Key size must be 16 bytes (128 bits)".into());
    }
    let block_size = <Aes128 as BlockSizeUser>::BlockSize::to_usize();
    if iv.len() != block_size {
        return Err("IV size must be 16 bytes (128 bits)".into());
    }
    if !data.len().is_multiple_of(block_size) {
        return Err("Data length must be a multiple of 16 bytes".into());
    }
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let key = GenericArray::from_slice(&key);
    let cipher = Aes128::new(key);

    let mut decrypted_data = data;
    let mut prev_block = iv;
    for chunk in decrypted_data.chunks_mut(block_size) {
        // Save the ciphertext block before decrypting in place, since CBC
        // chaining needs it to decrypt the *next* block.
        let ciphertext_block = chunk.to_vec();

        let block = GenericArray::from_mut_slice(chunk);
        cipher.decrypt_block(block);

        // CBC chaining: XOR the decrypted block with the previous ciphertext
        // block (or the IV, for the first block).
        for (byte, prev) in chunk.iter_mut().zip(prev_block.iter()) {
            *byte ^= prev;
        }

        prev_block = ciphertext_block;
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
        let iv = vec![0u8; 16];
        let data = b"Hello World!".to_vec();

        let encrypted = aes_encrypt(key.clone(), iv.clone(), data.clone()).unwrap();
        assert_ne!(encrypted, data);
        assert_eq!(encrypted.len() % 16, 0);

        let decrypted = aes_decrypt(key, iv, encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_aes_encrypt_decrypt_nonzero_iv() {
        let key = b"VERY SECRET KEY!".to_vec();
        let iv = b"0123456789ABCDEF".to_vec();
        let data = [
            [b'A'; 16].to_vec(),
            [b'A'; 16].to_vec(),
            b"tail bytes!".to_vec(),
        ]
        .concat();

        let encrypted = aes_encrypt(key.clone(), iv.clone(), data.clone()).unwrap();
        let decrypted = aes_decrypt(key, iv, encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_aes_decrypt_wrong_iv_fails_roundtrip() {
        let key = b"VERY SECRET KEY!".to_vec();
        let iv = vec![0u8; 16];
        let wrong_iv = vec![0xAAu8; 16];
        // Two full blocks: a wrong IV only corrupts the first block, so the
        // second block's plaintext (and thus the PKCS#7 padding, which lives
        // in the second block) is still recovered correctly, letting us
        // compare the full decrypted buffer.
        let data = [*b"AAAAAAAAAAAAAAAA", *b"BBBBBBBBBBBBBBBB"].concat();

        let encrypted = aes_encrypt(key.clone(), iv, data.clone()).unwrap();
        let decrypted = aes_decrypt(key, wrong_iv, encrypted).unwrap();
        // Only the first block is corrupted by a wrong IV.
        assert_ne!(decrypted, data);
        assert_eq!(&decrypted[16..], &data[16..]);
    }

    #[test]
    fn test_aes_decrypt_invalid_key() {
        let key = b"SHORT".to_vec();
        let iv = vec![0u8; 16];
        let data = vec![0u8; 16];
        assert!(aes_decrypt(key, iv, data).is_err());
    }

    #[test]
    fn test_aes_decrypt_invalid_iv() {
        let key = b"VERY SECRET KEY!".to_vec();
        let iv = vec![0u8; 8]; // Wrong size
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
