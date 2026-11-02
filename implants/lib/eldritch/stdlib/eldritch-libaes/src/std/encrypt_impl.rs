use aes::Aes128;
use aes::cipher::{
    BlockEncrypt, BlockSizeUser, KeyInit,
    generic_array::{GenericArray, typenum::Unsigned},
};
use alloc::string::String;
use alloc::vec::Vec;

pub fn aes_encrypt(key: Vec<u8>, _iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
    if key.len() != 16 {
        return Err("Key size must be 16 bytes (128 bits)".into());
    }
    let key = GenericArray::from_slice(&key);
    let cipher = Aes128::new(key);

    let block_size = <Aes128 as BlockSizeUser>::BlockSize::to_usize();
    // PKCS#7 padding: always add padding. If data is multiple of block_size, add full block of padding.
    let padding_len = block_size - (data.len() % block_size);
    let mut padded_data = data;
    padded_data.extend(core::iter::repeat_n(padding_len as u8, padding_len));

    for chunk in padded_data.chunks_mut(block_size) {
        let block = GenericArray::from_mut_slice(chunk);
        cipher.encrypt_block(block);
    }

    Ok(padded_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encrypt_invalid_key() {
        let key = b"SHORT".to_vec();
        let iv = vec![0u8; 16];
        let data = b"Hello".to_vec();
        assert!(aes_encrypt(key, iv, data).is_err());
    }
}
