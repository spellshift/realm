use super::decrypt_impl::validate_pkcs7_padding_ct;
use super::encrypt_file_impl::read_full_or_eof;
use aes::Aes128;
use aes::cipher::{BlockDecrypt, KeyInit, generic_array::GenericArray};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

/// Size of the AES block / IV, in bytes.
const BLOCK_SIZE: usize = 16;

/// Streaming buffer size (must be a non-zero multiple of `BLOCK_SIZE`).
const CHUNK_SIZE: usize = 64 * 1024;

pub fn aes_decrypt_file(src: String, dst: String, key: Vec<u8>) -> Result<(), String> {
    if key.len() != 16 {
        return Err("Key size must be 16 bytes".into());
    }

    let mut reader = BufReader::new(File::open(src).map_err(|e| e.to_string())?);

    // The IV was prepended to the ciphertext by `aes_encrypt_file`.
    let mut prev_block = [0u8; BLOCK_SIZE];
    if read_full_or_eof(&mut reader, &mut prev_block)? < BLOCK_SIZE {
        return Err("Encrypted file is too short to contain an IV".into());
    }

    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut n = read_full_or_eof(&mut reader, &mut buf)?;
    if n == 0 {
        // Empty ciphertext after IV: create empty destination file.
        File::create(dst).map_err(|e| e.to_string())?;
        return Ok(());
    }
    if n % BLOCK_SIZE != 0 {
        return Err("Data length must be a multiple of 16 bytes".into());
    }

    let mut writer = BufWriter::new(File::create(dst).map_err(|e| e.to_string())?);
    let cipher = Aes128::new(GenericArray::from_slice(&key));

    // Hold back one block so PKCS#7 unpadding is only applied to the true final block.
    let mut pending: Option<[u8; BLOCK_SIZE]> = None;

    loop {
        for chunk in buf[..n].chunks_exact(BLOCK_SIZE) {
            if let Some(mut ct_block) = pending.take() {
                let saved_ct = ct_block;
                let block = GenericArray::from_mut_slice(&mut ct_block);
                cipher.decrypt_block(block);
                for (byte, prev) in ct_block.iter_mut().zip(prev_block.iter()) {
                    *byte ^= prev;
                }
                prev_block = saved_ct;
                writer.write_all(&ct_block).map_err(|e| e.to_string())?;
            }
            let mut next_pending = [0u8; BLOCK_SIZE];
            next_pending.copy_from_slice(chunk);
            pending = Some(next_pending);
        }

        if n < CHUNK_SIZE {
            break;
        }
        n = read_full_or_eof(&mut reader, &mut buf)?;
        if n == 0 {
            break;
        }
        if n % BLOCK_SIZE != 0 {
            return Err("Data length must be a multiple of 16 bytes".into());
        }
    }

    let mut final_block = pending.ok_or_else(|| String::from("Encrypted file has no data"))?;
    let block = GenericArray::from_mut_slice(&mut final_block);
    cipher.decrypt_block(block);
    for (byte, prev) in final_block.iter_mut().zip(prev_block.iter()) {
        *byte ^= prev;
    }

    let pad_len = validate_pkcs7_padding_ct(&final_block)?;
    writer
        .write_all(&final_block[..BLOCK_SIZE - pad_len])
        .map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::encrypt_file_impl::aes_encrypt_file;
    use super::*;
    use std::io::{Read, Write};
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_file_encrypt_decrypt() {
        let mut src_file = NamedTempFile::new().unwrap();
        let data = b"Secret file content that needs encryption.";
        src_file.write_all(data).unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();

        let enc_file = NamedTempFile::new().unwrap();
        let enc_path = enc_file.path().to_str().unwrap().to_string();

        let dec_file = NamedTempFile::new().unwrap();
        let dec_path = dec_file.path().to_str().unwrap().to_string();

        let key = b"VERY SECRET KEY!".to_vec();

        aes_encrypt_file(src_path, enc_path.clone(), key.clone()).unwrap();

        let mut enc_content = Vec::new();
        File::open(&enc_path)
            .unwrap()
            .read_to_end(&mut enc_content)
            .unwrap();
        assert_ne!(enc_content, data);

        aes_decrypt_file(enc_path, dec_path.clone(), key).unwrap();

        let mut dec_content = Vec::new();
        File::open(&dec_path)
            .unwrap()
            .read_to_end(&mut dec_content)
            .unwrap();
        assert_eq!(dec_content, data);
    }

    #[test]
    fn test_aes_decrypt_file_errors() {
        let key_short = b"SHORT".to_vec();
        let key_valid = b"VERY SECRET KEY!".to_vec();

        // Short key
        assert!(aes_decrypt_file("src".to_string(), "dst".to_string(), key_short).is_err());

        // Non-existent source file
        assert!(
            aes_decrypt_file(
                "/non/existent/src".to_string(),
                "dst".to_string(),
                key_valid.clone()
            )
            .is_err()
        );

        // Invalid destination path
        let mut src_file = NamedTempFile::new().unwrap();
        let data = b"Some data";
        src_file.write_all(data).unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();

        let enc_file = NamedTempFile::new().unwrap();
        let enc_path = enc_file.path().to_str().unwrap().to_string();
        aes_encrypt_file(src_path, enc_path.clone(), key_valid.clone()).unwrap();

        assert!(
            aes_decrypt_file(enc_path, "/non/existent/dir/dst".to_string(), key_valid).is_err()
        );
    }

    #[test]
    fn test_aes_decrypt_file_too_short() {
        let key_valid = b"VERY SECRET KEY!".to_vec();

        let mut src_file = NamedTempFile::new().unwrap();
        // Shorter than a single IV.
        src_file.write_all(b"short").unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();

        assert!(aes_decrypt_file(src_path, "dst".to_string(), key_valid).is_err());
    }

    #[test]
    fn test_aes_file_streaming_roundtrip_multi_chunk_and_binary_key() {
        // Non-UTF-8 raw 16-byte key across multiple chunk sizes including exact boundaries.
        let key: Vec<u8> = (0xF0u8..=0xFF).collect();

        for size in [0usize, 1, 15, 16, 17, CHUNK_SIZE, CHUNK_SIZE * 2 + 13] {
            let data: Vec<u8> = (0..size).map(|i| ((i * 31) % 256) as u8).collect();
            let mut src_file = NamedTempFile::new().unwrap();
            src_file.write_all(&data).unwrap();
            let src_path = src_file.path().to_str().unwrap().to_string();

            let enc_file = NamedTempFile::new().unwrap();
            let enc_path = enc_file.path().to_str().unwrap().to_string();

            let dec_file = NamedTempFile::new().unwrap();
            let dec_path = dec_file.path().to_str().unwrap().to_string();

            aes_encrypt_file(src_path, enc_path.clone(), key.clone()).unwrap();
            aes_decrypt_file(enc_path, dec_path.clone(), key.clone()).unwrap();

            let mut dec_content = Vec::new();
            File::open(&dec_path)
                .unwrap()
                .read_to_end(&mut dec_content)
                .unwrap();
            assert_eq!(dec_content, data, "roundtrip mismatch for size {size}");
        }
    }

    #[test]
    fn test_aes_decrypt_file_corrupted_padding_fails() {
        let key = b"VERY SECRET KEY!".to_vec();
        let mut src_file = NamedTempFile::new().unwrap();
        src_file.write_all(b"Padding check payload").unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();

        let enc_file = NamedTempFile::new().unwrap();
        let enc_path = enc_file.path().to_str().unwrap().to_string();
        aes_encrypt_file(src_path, enc_path.clone(), key.clone()).unwrap();

        let mut enc_bytes = Vec::new();
        File::open(&enc_path)
            .unwrap()
            .read_to_end(&mut enc_bytes)
            .unwrap();
        let last = enc_bytes.len() - 1;
        enc_bytes[last] ^= 0xFF;
        File::create(&enc_path)
            .unwrap()
            .write_all(&enc_bytes)
            .unwrap();

        let dec_file = NamedTempFile::new().unwrap();
        let dec_path = dec_file.path().to_str().unwrap().to_string();
        assert!(aes_decrypt_file(enc_path, dec_path, key).is_err());
    }
}
