use super::AssetBackend;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use anyhow::Result;
#[allow(unused_imports)]
use chacha20poly1305::{
    XChaCha20Poly1305,
    aead::{Aead, KeyInit, generic_array::GenericArray},
};
use core::marker::PhantomData;
#[cfg(not(debug_assertions))]
use flate2::read::GzDecoder;
#[cfg(not(debug_assertions))]
use std::io::Read;

pub trait Embedable {
    fn get(file_path: &str) -> Option<Cow<'static, [u8]>>;
    fn iter() -> alloc::boxed::Box<dyn Iterator<Item = Cow<'static, str>>>;
    fn is_compressed() -> bool;
}

// An AssetBackend that gets assets from an Embedable and decrypts them
pub struct EmbeddedAssets<T: Embedable> {
    #[allow(dead_code)]
    key: Option<[u8; 32]>,
    _phantom: PhantomData<T>,
}

impl<T: Embedable> EmbeddedAssets<T> {
    pub fn new(key: Option<[u8; 32]>) -> Self {
        Self {
            key,
            _phantom: PhantomData,
        }
    }
}

impl<T: Embedable + Send + Sync + 'static> AssetBackend for EmbeddedAssets<T> {
    fn get(&self, name: &str) -> Result<Vec<u8>> {
        let data = T::get(name).ok_or_else(|| anyhow::anyhow!("asset not found: {}", name))?;

        #[cfg(debug_assertions)]
        {
            return Ok(data.into_owned());
        }

        #[cfg(not(debug_assertions))]
        {
            let maybe_compressed_data = if let Some(key) = self.key {
                // Expecting nonce (24 bytes) + ciphertext
                if data.len() < 24 {
                    return Err(anyhow::anyhow!("Asset too short to be encrypted"));
                }

                let nonce = GenericArray::from_slice(&data[0..24]);
                let ciphertext = &data[24..];

                let cipher = XChaCha20Poly1305::new(GenericArray::from_slice(&key));

                cipher
                    .decrypt(nonce, ciphertext)
                    .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?
            } else {
                // No key, assume just payload
                data.into_owned()
            };

            if T::is_compressed() {
                let mut decoder = GzDecoder::new(&maybe_compressed_data[..]);
                let mut decompressed = Vec::new();
                decoder
                    .read_to_end(&mut decompressed)
                    .map_err(|e| anyhow::anyhow!("Decompression failed: {}", e))?;
                Ok(decompressed)
            } else {
                Ok(maybe_compressed_data)
            }
        }
    }

    fn assets(&self) -> Vec<Cow<'static, str>> {
        T::iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock Embed implementation
    struct MockEmbed;

    // Static storage for mock assets
    static MOCK_ASSETS: Lazy<Mutex<HashMap<String, Vec<u8>>>> =
        Lazy::new(|| Mutex::new(HashMap::new()));

    impl Embedable for MockEmbed {
        fn get(file_path: &str) -> Option<Cow<'static, [u8]>> {
            let assets = MOCK_ASSETS.lock().unwrap();
            assets.get(file_path).map(|data| Cow::Owned(data.clone()))
        }

        fn iter() -> alloc::boxed::Box<dyn Iterator<Item = std::borrow::Cow<'static, str>>> {
            alloc::boxed::Box::new(std::iter::empty())
        }

        fn is_compressed() -> bool {
            true // Default mock behavior is compressed
        }
    }

    #[cfg(not(debug_assertions))]
    struct MockEmbedNoCompress;

    #[cfg(not(debug_assertions))]
    impl Embedable for MockEmbedNoCompress {
        fn get(file_path: &str) -> Option<Cow<'static, [u8]>> {
            let assets = MOCK_ASSETS.lock().unwrap();
            assets.get(file_path).map(|data| Cow::Owned(data.clone()))
        }

        fn iter() -> alloc::boxed::Box<dyn Iterator<Item = std::borrow::Cow<'static, str>>> {
            alloc::boxed::Box::new(std::iter::empty())
        }

        fn is_compressed() -> bool {
            false
        }
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn test_no_compression_flag() {
        use chacha20poly1305::{
            XChaCha20Poly1305,
            aead::{Aead, KeyInit, generic_array::GenericArray},
        };
        use rand::{RngCore, SeedableRng};
        use rand_chacha::ChaCha20Rng;

        let mut rng = ChaCha20Rng::from_entropy();
        let mut key = [0u8; 32];
        rng.fill_bytes(&mut key);

        let plaintext = b"Hello, world! This is UNCOMPRESSED but ENCRYPTED.";

        // NO Compress step!

        // Encrypt manually
        let cipher = XChaCha20Poly1305::new(GenericArray::from_slice(&key));
        let mut nonce_bytes = [0u8; 24];
        rng.fill_bytes(&mut nonce_bytes);
        let nonce = GenericArray::from_slice(&nonce_bytes);

        // Encrypt the PLAINTEXT directly
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .expect("Encryption failed");

        let mut encrypted_asset = Vec::new();
        encrypted_asset.extend_from_slice(&nonce_bytes);
        encrypted_asset.extend_from_slice(&ciphertext);

        // Store
        MOCK_ASSETS
            .lock()
            .unwrap()
            .insert("test_no_compress_flag.txt".to_string(), encrypted_asset);

        // Test retrieval via EmbeddedAssets with MockEmbedNoCompress
        let assets = EmbeddedAssets::<MockEmbedNoCompress>::new(Some(key));
        let retrieved = assets
            .get("test_no_compress_flag.txt")
            .expect("Retrieval failed");

        assert_eq!(
            retrieved, plaintext,
            "Data mismatch: Expected uncompressed plaintext"
        );
    }

    #[test]
    #[cfg(not(debug_assertions))] // This test only makes sense when decryption is active
    fn test_encryption_roundtrip() {
        use chacha20poly1305::{
            XChaCha20Poly1305,
            aead::{Aead, KeyInit, generic_array::GenericArray},
        };
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use rand::{RngCore, SeedableRng};
        use rand_chacha::ChaCha20Rng;
        use std::io::Write;

        let mut rng = ChaCha20Rng::from_entropy();
        let mut key = [0u8; 32];
        rng.fill_bytes(&mut key);

        let plaintext = b"Hello, world! This is a test asset.";

        // Compress
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(plaintext).expect("Compression failed");
        let compressed_plaintext = encoder.finish().expect("Compression finish failed");

        // Encrypt manually
        let cipher = XChaCha20Poly1305::new(GenericArray::from_slice(&key));
        let mut nonce_bytes = [0u8; 24];
        rng.fill_bytes(&mut nonce_bytes);
        let nonce = GenericArray::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, compressed_plaintext.as_ref())
            .expect("Encryption failed");

        let mut encrypted_asset = Vec::new();
        encrypted_asset.extend_from_slice(&nonce_bytes);
        encrypted_asset.extend_from_slice(&ciphertext);

        // Store in mock assets
        MOCK_ASSETS
            .lock()
            .unwrap()
            .insert("test_asset.txt".to_string(), encrypted_asset);

        // Test decryption via EmbeddedAssets
        let assets = EmbeddedAssets::<MockEmbed>::new(Some(key));
        let decrypted = assets
            .get("test_asset.txt")
            .expect("Decryption via asset backend failed");

        assert_eq!(
            decrypted, plaintext,
            "Decrypted data does not match original plaintext"
        );
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn test_no_encryption_roundtrip() {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;

        let plaintext = b"Hello, world! This is an UNENCRYPTED test asset.";

        // Compress
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(plaintext).expect("Compression failed");
        let compressed_plaintext = encoder.finish().expect("Compression finish failed");

        // Store directly (Compressed only)
        MOCK_ASSETS
            .lock()
            .unwrap()
            .insert("test_unencrypted.txt".to_string(), compressed_plaintext);

        // Test retrieval via EmbeddedAssets with NO key
        let assets = EmbeddedAssets::<MockEmbed>::new(None);
        let decompressed = assets
            .get("test_unencrypted.txt")
            .expect("Retrieval via asset backend failed");

        assert_eq!(
            decompressed, plaintext,
            "Decompressed data does not match original plaintext"
        );
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_debug_passthrough() {
        let plaintext = b"Hello, world! This is a debug test.";
        // Key doesn't matter in debug

        // Store plaintext in mock assets
        MOCK_ASSETS
            .lock()
            .unwrap()
            .insert("debug_asset.txt".to_string(), plaintext.to_vec());

        // Test passthrough
        let assets = EmbeddedAssets::<MockEmbed>::new(None);
        let retrieved = assets.get("debug_asset.txt").expect("Retrieval failed");

        assert_eq!(
            retrieved, plaintext,
            "Data should be passed through in debug mode"
        );
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn test_random_strings_integrity() {
        use chacha20poly1305::{
            XChaCha20Poly1305,
            aead::{Aead, KeyInit, generic_array::GenericArray},
        };
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use rand::distributions::Alphanumeric;
        use rand::{Rng, thread_rng};
        use rand::{RngCore, SeedableRng};
        use rand_chacha::ChaCha20Rng;
        use sha2::{Digest, Sha256};
        use std::io::Write;

        let mut chacha_rng = ChaCha20Rng::from_entropy();
        let mut key = [0u8; 32];
        chacha_rng.fill_bytes(&mut key);

        let assets_backend = EmbeddedAssets::<MockEmbed>::new(Some(key));
        let cipher = XChaCha20Poly1305::new(GenericArray::from_slice(&key));

        for i in 0..100 {
            // Generate random string
            let rand_string: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(30 + i) // Varied length
                .map(char::from)
                .collect();
            let plaintext = rand_string.as_bytes();

            // Calculate Hash of original
            let mut hasher = Sha256::new();
            hasher.update(plaintext);
            let original_hash = hasher.finalize();

            // Compress
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(plaintext).expect("Compression failed");
            let compressed_plaintext = encoder.finish().expect("Compression finish failed");

            // Encrypt
            let mut nonce_bytes = [0u8; 24];
            chacha_rng.fill_bytes(&mut nonce_bytes);
            let nonce = GenericArray::from_slice(&nonce_bytes);
            let ciphertext = cipher
                .encrypt(nonce, compressed_plaintext.as_ref())
                .expect("Encryption failed");

            // Pack (nonce + ciphertext)
            let mut encrypted_asset = Vec::new();
            encrypted_asset.extend_from_slice(&nonce_bytes);
            encrypted_asset.extend_from_slice(&ciphertext);

            // Store in Mock
            let asset_name = format!("random_asset_{}.txt", i);
            MOCK_ASSETS
                .lock()
                .unwrap()
                .insert(asset_name.clone(), encrypted_asset);

            // Retrieve and Decrypt
            let decrypted = assets_backend
                .get(&asset_name)
                .expect("Failed to retrieve asset");

            // Calculate Hash of decrypted
            let mut hasher = Sha256::new();
            hasher.update(&decrypted);
            let decrypted_hash = hasher.finalize();

            // Verify
            assert_eq!(decrypted, plaintext, "Content mismatch for asset {}", i);
            assert_eq!(
                decrypted_hash, original_hash,
                "Hash mismatch for asset {}",
                i
            );
        }
    }
}
