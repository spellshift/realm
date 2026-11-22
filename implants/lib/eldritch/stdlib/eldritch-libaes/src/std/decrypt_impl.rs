use aes::Aes128;
use aes::cipher::{
    BlockDecrypt, BlockSizeUser, KeyInit,
    generic_array::{GenericArray, typenum::Unsigned},
};
use alloc::string::String;
use alloc::vec::Vec;

/// Validates PKCS#7 padding on the final 16-byte decrypted block in constant time.
///
/// Inspects every byte in `block` without data-dependent early returns so
/// timing cannot distinguish invalid padding length from an invalid padding byte.
pub(crate) fn validate_pkcs7_padding_ct(block: &[u8; 16]) -> Result<usize, String> {
    let pad_len = block[15];
    // Valid PKCS#7 pad_len is in 1..=16.
    // `pad_len.wrapping_sub(1)` is in 0..=15 iff pad_len is in 1..=16 (upper 4 bits are 0).
    let mut bad: u8 = pad_len.wrapping_sub(1) & 0xF0;

    for (i, &b) in block.iter().enumerate() {
        let dist = (16 - i) as u8;
        // If dist <= pad_len, subtraction does not borrow -> high byte is 0x00 -> mask is 0xFF.
        // If dist > pad_len, subtraction borrows -> high byte is 0xFF -> mask is 0x00.
        let out_of_pad = ((pad_len as u16).wrapping_sub(dist as u16) >> 8) as u8;
        let in_pad_mask = !out_of_pad;
        bad |= (b ^ pad_len) & in_pad_mask;
    }

    if core::hint::black_box(bad) != 0 {
        return Err("Invalid padding".into());
    }
    Ok(pad_len as usize)
}

/// Decrypts `data` with AES-128 in CBC mode, removing PKCS#7 padding.
///
/// `key` must be exactly 16 bytes and `iv` must be exactly 16 bytes (the AES
/// block size). The IV is mixed into every block via CBC chaining; it is not
/// ignored.
///
/// Note: AES-CBC is unauthenticated (no integrity tag / MAC).
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

    // Constant-time PKCS#7 unpadding over the final 16-byte block.
    let last_block_start = decrypted_data.len() - block_size;
    let last_block: &[u8; 16] = decrypted_data[last_block_start..]
        .try_into()
        .map_err(|_| String::from("Decryption failed: invalid final block size"))?;
    let pad_len = validate_pkcs7_padding_ct(last_block)?;
    decrypted_data.truncate(decrypted_data.len() - pad_len);
    Ok(decrypted_data)
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

    /// Decodes a hex string (e.g. `"DEADBEEF"`) into bytes, for use with
    /// published test vectors.
    fn decode_hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    /// Known-answer test against the NIST SP 800-38A (2001) Appendix F.2.1
    /// "CBC-AES128.Encrypt" test vector (verified independently with
    /// `openssl enc -aes-128-cbc -nopad`), extended with one trailing
    /// PKCS#7 padding block so it round-trips through our padded
    /// `aes_decrypt`/`aes_encrypt` API: the ciphertext here is the NIST
    /// vector's 4 blocks followed by `openssl`'s encryption of a
    /// `[0x10; 16]` PKCS#7 padding block chained after them.
    #[test]
    fn test_aes_decrypt_matches_nist_sp800_38a_f_2_1_vector() {
        let key = decode_hex("2B7E151628AED2A6ABF7158809CF4F3C");
        let iv = decode_hex("000102030405060708090A0B0C0D0E0F");
        let padded_ciphertext = decode_hex(
            "7649ABAC8119B246CEE98E9B12E9197D5086CB9B507219EE95DB113A917678B273BED6B8E3C1743B7116E69E222295163FF1CAA1681FAC09120ECA307586E1A78CB82807230E1321D3FAE00D18CC2012",
        );
        let expected_plaintext = decode_hex(
            "6BC1BEE22E409F96E93D7E117393172AAE2D8A571E03AC9C9EB76FAC45AF8E5130C81C46A35CE411E5FBC1191A0A52EFF69F2445DF4F9B17AD2B417BE66C3710",
        );

        let decrypted = aes_decrypt(key, iv, padded_ciphertext).unwrap();
        assert_eq!(decrypted, expected_plaintext);
    }

    #[test]
    fn test_validate_pkcs7_padding_ct_all_lengths_and_invalid() {
        // Valid padding for every length 1..=16.
        for pad_len in 1u8..=16 {
            let mut block = [0xAAu8; 16];
            for b in &mut block[16 - pad_len as usize..] {
                *b = pad_len;
            }
            assert_eq!(validate_pkcs7_padding_ct(&block).unwrap(), pad_len as usize);

            // Corrupting any padding byte must fail.
            for idx in (16 - pad_len as usize)..16 {
                let mut corrupted = block;
                corrupted[idx] ^= 0x01;
                assert!(validate_pkcs7_padding_ct(&corrupted).is_err());
            }
        }

        // Invalid pad lengths: 0, 17, 255.
        for bad_len in [0u8, 17, 255] {
            let block = [bad_len; 16];
            assert!(validate_pkcs7_padding_ct(&block).is_err());
        }
    }
}
