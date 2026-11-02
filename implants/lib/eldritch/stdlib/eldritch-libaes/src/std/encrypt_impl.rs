use aes::Aes128;
use aes::cipher::{
    BlockEncrypt, BlockSizeUser, KeyInit,
    generic_array::{GenericArray, typenum::Unsigned},
};
use alloc::string::String;
use alloc::vec::Vec;

/// Encrypts `data` with AES-128 in CBC mode using PKCS#7 padding.
///
/// `key` must be exactly 16 bytes and `iv` must be exactly 16 bytes (the AES
/// block size). The IV is mixed into every block via CBC chaining; it is not
/// ignored.
pub fn aes_encrypt(key: Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, String> {
    if key.len() != 16 {
        return Err("Key size must be 16 bytes (128 bits)".into());
    }
    let block_size = <Aes128 as BlockSizeUser>::BlockSize::to_usize();
    if iv.len() != block_size {
        return Err("IV size must be 16 bytes (128 bits)".into());
    }

    let key = GenericArray::from_slice(&key);
    let cipher = Aes128::new(key);

    // PKCS#7 padding: always add padding. If data is multiple of block_size, add full block of padding.
    let padding_len = block_size - (data.len() % block_size);
    let mut padded_data = data;
    padded_data.extend(core::iter::repeat_n(padding_len as u8, padding_len));

    let mut prev_block = iv;
    for chunk in padded_data.chunks_mut(block_size) {
        // CBC chaining: XOR the plaintext block with the previous ciphertext
        // block (or the IV, for the first block) before encrypting.
        for (byte, prev) in chunk.iter_mut().zip(prev_block.iter()) {
            *byte ^= prev;
        }
        let block = GenericArray::from_mut_slice(chunk);
        cipher.encrypt_block(block);
        prev_block.copy_from_slice(chunk);
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

    #[test]
    fn test_aes_encrypt_invalid_iv() {
        let key = b"VERY SECRET KEY!".to_vec();
        let iv = vec![0u8; 8]; // Wrong size
        let data = b"Hello".to_vec();
        assert!(aes_encrypt(key, iv, data).is_err());
    }

    #[test]
    fn test_aes_encrypt_iv_changes_ciphertext() {
        let key = b"VERY SECRET KEY!".to_vec();
        let data = b"Hello World!".to_vec();

        let iv_a = vec![0u8; 16];
        let iv_b = vec![0xABu8; 16];

        let encrypted_a = aes_encrypt(key.clone(), iv_a, data.clone()).unwrap();
        let encrypted_b = aes_encrypt(key, iv_b, data).unwrap();

        assert_ne!(
            encrypted_a, encrypted_b,
            "different IVs must produce different ciphertext"
        );
    }

    #[test]
    fn test_aes_encrypt_cbc_hides_repeated_blocks() {
        let key = b"VERY SECRET KEY!".to_vec();
        let iv = vec![0x11u8; 16];
        // Two identical 16-byte plaintext blocks.
        let data = [[b'A'; 16], [b'A'; 16]].concat();

        let encrypted = aes_encrypt(key, iv, data).unwrap();
        let first_block = &encrypted[0..16];
        let second_block = &encrypted[16..32];

        assert_ne!(
            first_block, second_block,
            "CBC chaining must prevent identical plaintext blocks from producing identical ciphertext blocks"
        );
    }

    /// Decodes a hex string (e.g. `"DEADBEEF"`) into bytes, for use with
    /// published test vectors.
    fn decode_hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    /// Known-answer test against the NIST SP 800-38A (2001) Appendix F.2.1
    /// "CBC-AES128.Encrypt" test vector, cross-checked independently with
    /// `openssl enc -aes-128-cbc -nopad`.
    ///
    /// The vector itself is raw CBC with no padding, so only the first 4
    /// blocks (64 bytes) of our output are comparable: `aes_encrypt` always
    /// appends a full PKCS#7 padding block since the 64-byte input is
    /// already block-aligned.
    #[test]
    fn test_aes_encrypt_matches_nist_sp800_38a_f_2_1_vector() {
        let key = decode_hex("2B7E151628AED2A6ABF7158809CF4F3C");
        let iv = decode_hex("000102030405060708090A0B0C0D0E0F");
        let plaintext = decode_hex(
            "6BC1BEE22E409F96E93D7E117393172AAE2D8A571E03AC9C9EB76FAC45AF8E5130C81C46A35CE411E5FBC1191A0A52EFF69F2445DF4F9B17AD2B417BE66C3710",
        );
        let expected_ciphertext = decode_hex(
            "7649ABAC8119B246CEE98E9B12E9197D5086CB9B507219EE95DB113A917678B273BED6B8E3C1743B7116E69E222295163FF1CAA1681FAC09120ECA307586E1A7",
        );

        let encrypted = aes_encrypt(key, iv, plaintext).unwrap();
        assert_eq!(&encrypted[..64], expected_ciphertext.as_slice());
    }
}
