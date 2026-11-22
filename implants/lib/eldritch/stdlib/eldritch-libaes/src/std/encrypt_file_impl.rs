use aes::Aes128;
use aes::cipher::{BlockEncrypt, KeyInit, generic_array::GenericArray};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

/// Size of the AES block / IV, in bytes.
const BLOCK_SIZE: usize = 16;

/// Streaming buffer size (must be a non-zero multiple of `BLOCK_SIZE`).
const CHUNK_SIZE: usize = 64 * 1024;

/// Reads into `buf` until either `buf` is completely full or EOF is reached.
pub(crate) fn read_full_or_eof<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<usize, String> {
    let mut total = 0;
    while total < buf.len() {
        let n = reader.read(&mut buf[total..]).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        total += n;
    }
    Ok(total)
}

pub fn aes_encrypt_file(src: String, dst: String, key: Vec<u8>) -> Result<(), String> {
    if key.len() != 16 {
        return Err("Key size must be 16 bytes".into());
    }

    let mut reader = BufReader::new(File::open(src).map_err(|e| e.to_string())?);
    let mut writer = BufWriter::new(File::create(dst).map_err(|e| e.to_string())?);

    let cipher = Aes128::new(GenericArray::from_slice(&key));

    // Generate a fresh random IV for every file so that encrypting the same
    // plaintext twice does not leak information via identical ciphertext.
    let mut prev_block = [0u8; BLOCK_SIZE];
    rand_chacha::ChaCha20Rng::from_entropy().fill_bytes(&mut prev_block);

    // Prepend the IV to the ciphertext so `aes_decrypt_file` can recover it.
    writer.write_all(&prev_block).map_err(|e| e.to_string())?;

    let mut buf = vec![0u8; CHUNK_SIZE];
    loop {
        let n = read_full_or_eof(&mut reader, &mut buf)?;
        let full_bytes = n - (n % BLOCK_SIZE);

        for chunk in buf[..full_bytes].chunks_mut(BLOCK_SIZE) {
            for (byte, prev) in chunk.iter_mut().zip(prev_block.iter()) {
                *byte ^= prev;
            }
            let block = GenericArray::from_mut_slice(chunk);
            cipher.encrypt_block(block);
            prev_block.copy_from_slice(chunk);
        }

        if full_bytes > 0 {
            writer
                .write_all(&buf[..full_bytes])
                .map_err(|e| e.to_string())?;
        }

        if n < CHUNK_SIZE {
            // Reached EOF: build final PKCS#7 padded block (always appended, even if rem == 0).
            let rem = n - full_bytes;
            let pad_byte = (BLOCK_SIZE - rem) as u8;
            let mut final_block = [pad_byte; BLOCK_SIZE];
            final_block[..rem].copy_from_slice(&buf[full_bytes..n]);

            for (byte, prev) in final_block.iter_mut().zip(prev_block.iter()) {
                *byte ^= prev;
            }
            let block = GenericArray::from_mut_slice(&mut final_block);
            cipher.encrypt_block(block);
            writer.write_all(&final_block).map_err(|e| e.to_string())?;
            break;
        }
    }

    writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::decrypt_impl::aes_decrypt;
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_aes_encrypt_file_errors() {
        let key_short = b"SHORT".to_vec();
        let key_valid = b"VERY SECRET KEY!".to_vec();

        // Short key
        assert!(aes_encrypt_file("src".to_string(), "dst".to_string(), key_short).is_err());

        // Non-existent source file
        assert!(
            aes_encrypt_file(
                "/non/existent/src".to_string(),
                "dst".to_string(),
                key_valid.clone()
            )
            .is_err()
        );

        // Invalid destination path
        let mut src_file = NamedTempFile::new().unwrap();
        src_file.write_all(b"test").unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();

        assert!(
            aes_encrypt_file(src_path, "/non/existent/dir/dst".to_string(), key_valid).is_err()
        );
    }

    #[test]
    fn test_aes_encrypt_file_random_iv() {
        let key = b"VERY SECRET KEY!".to_vec();

        let mut src_file = NamedTempFile::new().unwrap();
        src_file.write_all(b"Same plaintext twice.").unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();

        let enc_file_a = NamedTempFile::new().unwrap();
        let enc_path_a = enc_file_a.path().to_str().unwrap().to_string();
        let enc_file_b = NamedTempFile::new().unwrap();
        let enc_path_b = enc_file_b.path().to_str().unwrap().to_string();

        aes_encrypt_file(src_path.clone(), enc_path_a.clone(), key.clone()).unwrap();
        aes_encrypt_file(src_path, enc_path_b.clone(), key).unwrap();

        let mut content_a = Vec::new();
        File::open(&enc_path_a)
            .unwrap()
            .read_to_end(&mut content_a)
            .unwrap();
        let mut content_b = Vec::new();
        File::open(&enc_path_b)
            .unwrap()
            .read_to_end(&mut content_b)
            .unwrap();

        assert_ne!(
            content_a, content_b,
            "encrypting the same file twice should use a fresh random IV each time"
        );
    }

    #[test]
    fn test_aes_encrypt_file_binary_key_and_chunk_boundaries() {
        // Non-UTF-8 raw 16-byte key.
        let key: Vec<u8> = (0x80u8..0x90).collect();

        for size in [0usize, 15, 16, 17, CHUNK_SIZE, CHUNK_SIZE + 19] {
            let plaintext: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
            let mut src_file = NamedTempFile::new().unwrap();
            src_file.write_all(&plaintext).unwrap();
            let src_path = src_file.path().to_str().unwrap().to_string();

            let enc_file = NamedTempFile::new().unwrap();
            let enc_path = enc_file.path().to_str().unwrap().to_string();

            aes_encrypt_file(src_path, enc_path.clone(), key.clone()).unwrap();

            let mut encrypted = Vec::new();
            File::open(&enc_path)
                .unwrap()
                .read_to_end(&mut encrypted)
                .unwrap();
            let (iv, ct) = encrypted.split_at(BLOCK_SIZE);
            let decrypted = aes_decrypt(key.clone(), iv.to_vec(), ct.to_vec()).unwrap();
            assert_eq!(decrypted, plaintext, "mismatch for size {size}");
        }
    }
}
