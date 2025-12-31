use anyhow::{Context, Result};
use bytes::{Buf, BufMut};
use chacha20poly1305::{
    aead::{generic_array::GenericArray, AeadInPlace, OsRng},
    AeadCore, KeyInit,
};
#[cfg(feature = "imix")]
use const_decoder::Decoder as const_decode;
use lru::LruCache;
use prost::Message;
use rand_chacha::rand_core::SeedableRng;
use std::{
    io::{Read, Write},
    marker::PhantomData,
    num::NonZeroUsize,
    sync::{Mutex, OnceLock},
};
use tonic::{
    codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder},
    Status,
};
use x25519_dalek::{EphemeralSecret, PublicKey};

/* Compile-time constant for the server pubkey, derived from the IMIX_SERVER_PUBKEY environment variable during compilation.
 * To find the servers pubkey check the startup messages on the server look for `[INFO] Public key: <SERVER_PUBKEY>`
 */
#[cfg(feature = "imix")]
static SERVER_PUBKEY: [u8; 32] = const_decode::Base64.decode(env!("IMIX_SERVER_PUBKEY").as_bytes());

#[cfg(not(feature = "imix"))]
static SERVER_PUBKEY: [u8; 32] = [
    165, 30, 122, 188, 50, 89, 111, 214, 247, 4, 189, 217, 188, 37, 200, 190, 2, 180, 175, 107,
    194, 147, 177, 98, 103, 84, 99, 120, 72, 73, 87, 37,
];
// ------------

const KEY_CACHE_SIZE: usize = 1024;
const PUBKEY_LEN: usize = 32;
const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 16;
// Header is 56 bytes (32+24). To align payload to 16 bytes, we need offset 8 (56+8=64).
const ALIGN_PAD: usize = 8;

// The client and server may have multiple connections open and they may not resolve in order or sequentially.
// To handle this ephemeral public keys and shared secrets are stored in a hashmap key_history where they can be looked up
// by public key.
fn key_history() -> &'static Mutex<LruCache<[u8; 32], [u8; 32]>> {
    static ARRAY: OnceLock<Mutex<LruCache<[u8; 32], [u8; 32]>>> = OnceLock::new();
    ARRAY.get_or_init(|| Mutex::new(LruCache::new(NonZeroUsize::new(KEY_CACHE_SIZE).unwrap())))
}

fn add_key_history(pub_key: [u8; 32], shared_secret: [u8; 32]) {
    key_history().lock().unwrap().put(pub_key, shared_secret); // Mutex's must unwrap
}

fn get_key(pub_key: [u8; 32]) -> Result<[u8; 32]> {
    // Lookup the shared secret based on the public key
    let res = *key_history()
        .lock()
        .unwrap() // Mutex's must unwrap
        .get(&pub_key)
        .context("Unable to find shared secret for the public key recieved")?;
    Ok(res)
}

// Internal optimized encryption helper
// Appends Header + Encrypted(pt) + Tag to `out`.
// `out` must have enough capacity or it will grow.
#[allow(dead_code)]
fn encrypt_into_vec(pt: &[u8], out: &mut Vec<u8>) -> Result<()> {
    // Store server pubkey
    let server_public: PublicKey = PublicKey::from(SERVER_PUBKEY);

    // Generate ephemeral keys
    let rng = rand_chacha::ChaCha20Rng::from_entropy();
    let client_secret = EphemeralSecret::random_from_rng(rng);
    let client_public = PublicKey::from(&client_secret);

    // Generate shared secret
    let shared_secret = client_secret.diffie_hellman(&server_public);
    add_key_history(*client_public.as_bytes(), *shared_secret.as_bytes());

    // Generate nonce and cipher
    let cipher = chacha20poly1305::XChaCha20Poly1305::new(GenericArray::from_slice(
        shared_secret.as_bytes(),
    ));
    let nonce = chacha20poly1305::XChaCha20Poly1305::generate_nonce(&mut OsRng);

    // Write Header
    out.extend_from_slice(client_public.as_bytes());
    out.extend_from_slice(nonce.as_slice());

    // Write Plaintext
    let pt_start = out.len();
    out.extend_from_slice(pt);

    // Encrypt in place detached
    let tag = cipher
        .encrypt_in_place_detached(&nonce, &[], &mut out[pt_start..])
        .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

    // Append Tag
    out.extend_from_slice(tag.as_slice());

    Ok(())
}

// Optimized helper for encoding and encrypting directly into a buffer
// Uses padding for alignment.
fn encode_encrypt_into_buf<T>(msg: &T, buf: &mut EncodeBuf<'_>) -> Result<()>
where
    T: Message,
{
    // Calculate total size to reserve
    let encoded_len = msg.encoded_len();
    let total_len = PUBKEY_LEN + NONCE_LEN + encoded_len + TAG_LEN;

    // Allocate with padding for alignment
    let mut buffer = Vec::with_capacity(ALIGN_PAD + total_len);
    buffer.extend_from_slice(&[0u8; ALIGN_PAD]);

    // Store server pubkey
    let server_public: PublicKey = PublicKey::from(SERVER_PUBKEY);
    let rng = rand_chacha::ChaCha20Rng::from_entropy();
    let client_secret = EphemeralSecret::random_from_rng(rng);
    let client_public = PublicKey::from(&client_secret);
    let shared_secret = client_secret.diffie_hellman(&server_public);
    add_key_history(*client_public.as_bytes(), *shared_secret.as_bytes());

    let cipher = chacha20poly1305::XChaCha20Poly1305::new(GenericArray::from_slice(
        shared_secret.as_bytes(),
    ));
    let nonce = chacha20poly1305::XChaCha20Poly1305::generate_nonce(&mut OsRng);

    // Write Header (at offset ALIGN_PAD)
    buffer.extend_from_slice(client_public.as_bytes());
    buffer.extend_from_slice(nonce.as_slice());

    // Write Plaintext (Encode directly into buffer).
    // Start position will be ALIGN_PAD + 56 = 64 (Aligned!)
    msg.encode(&mut buffer).map_err(|e| anyhow::anyhow!("Protobuf encode failed: {:?}", e))?;

    // Encrypt in place detached
    // Plaintext is at [ALIGN_PAD + PUBKEY_LEN + NONCE_LEN .. ]
    let pt_start = ALIGN_PAD + PUBKEY_LEN + NONCE_LEN;
    let tag = cipher
        .encrypt_in_place_detached(&nonce, &[], &mut buffer[pt_start..])
        .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

    // Append Tag
    buffer.extend_from_slice(tag.as_slice());

    // Write to EncodeBuf (skipping padding)
    buf.writer().write_all(&buffer[ALIGN_PAD..]).map_err(|e| anyhow::anyhow!("Write failed: {:?}", e))?;

    Ok(())
}

// Helper to decrypt in place within a mutable buffer.
// Returns the length of the plaintext.
// The plaintext will start at PUBKEY_LEN + NONCE_LEN in the buffer.
fn decrypt_in_place_buf(buf: &mut [u8]) -> Result<usize> {
    if buf.is_empty() {
        return Ok(0);
    }
    if buf.len() < PUBKEY_LEN + NONCE_LEN + TAG_LEN {
        anyhow::bail!("Message too small to contain header and tag");
    }

    // Extract components (header)
    let client_public = &buf[0..PUBKEY_LEN];
    let nonce_slice = &buf[PUBKEY_LEN..PUBKEY_LEN + NONCE_LEN];

    // Copy necessary header data to avoid immutable borrow while mutating
    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(nonce_slice);

    let mut pub_key = [0u8; PUBKEY_LEN];
    pub_key.copy_from_slice(client_public);

    let client_private_bytes = get_key(pub_key).map_err(from_anyhow_error)?;

    let cipher =
        chacha20poly1305::XChaCha20Poly1305::new(GenericArray::from_slice(&client_private_bytes));

    // Split buffer to separate Tag
    let len = buf.len();
    let tag_start = len - TAG_LEN;
    let (ct_buf, tag_buf) = buf.split_at_mut(tag_start);

    // ct_buf contains Header + Ciphertext
    // Ciphertext starts at PUBKEY_LEN + NONCE_LEN
    let pt_start = PUBKEY_LEN + NONCE_LEN;
    let ct_slice = &mut ct_buf[pt_start..];

    let nonce_ga = GenericArray::from_slice(&nonce);
    let tag_ga = GenericArray::from_slice(tag_buf);

    cipher.decrypt_in_place_detached(nonce_ga, &[], ct_slice, tag_ga)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;

    Ok(ct_slice.len())
}

// ------------

#[derive(Debug, Clone, Default)]
pub struct ChaChaSvc {}

#[derive(Debug, Clone)]
pub struct ChachaCodec<T, U>(PhantomData<(T, U)>, ChaChaSvc);

impl<T, U> Default for ChachaCodec<T, U> {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        log::debug!("Loaded custom codec with xchacha encryption");
        Self(PhantomData, ChaChaSvc::default())
    }
}

impl<T, U> Codec for ChachaCodec<T, U>
where
    T: Message + Send + 'static,
    U: Message + Default + Send + 'static,
{
    type Encode = T;
    type Decode = U;
    type Encoder = ChachaEncrypt<T, U>;
    type Decoder = ChachaDecrypt<T, U>;

    fn encoder(&mut self) -> Self::Encoder {
        ChachaEncrypt(PhantomData, ChaChaSvc::default())
    }

    fn decoder(&mut self) -> Self::Decoder {
        ChachaDecrypt(PhantomData, ChaChaSvc::default())
    }
}

// ---

#[derive(Debug)]
pub struct ChachaEncrypt<T, U>(PhantomData<(T, U)>, ChaChaSvc);

impl<T, U> Encoder for ChachaEncrypt<T, U>
where
    T: Message + Send + 'static,
    U: Message + Default + Send + 'static,
{
    type Item = T;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, buf: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        if !buf.has_remaining_mut() {
            #[cfg(debug_assertions)]
            log::debug!("DANGER can't add to the buffer.");
        }

        // Use optimized path with alignment
        encode_encrypt_into_buf(&item, buf).map_err(|e| Status::new(tonic::Code::Internal, e.to_string()))
    }
}

// ---
//

#[derive(Debug)]
pub struct ChachaDecrypt<T, U>(PhantomData<(T, U)>, ChaChaSvc);

impl<T, U> Decoder for ChachaDecrypt<T, U>
where
    T: Message + Send + 'static,
    U: Message + Default + Send + 'static,
{
    type Item = U;
    type Error = Status;

    fn decode(&mut self, buf: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        // public key + xchacha nonce + ciphertext

        // Pre-allocate buffer to avoid reallocations
        let frame_len = buf.remaining();
        if frame_len == 0 {
             return Ok(None);
        }

        // Use padding for alignment
        let mut bytes_in = Vec::with_capacity(ALIGN_PAD + frame_len);
        bytes_in.extend_from_slice(&[0u8; ALIGN_PAD]);

        let mut reader = buf.reader();
        let bytes_read = match reader.read_to_end(&mut bytes_in) {
            Ok(n) => n,
            Err(err) => {
                #[cfg(debug_assertions)]
                log::debug!(
                    "decode error unable to read bytes from decode reader: {:?}",
                    err
                );
                return Err(Status::new(tonic::Code::Internal, err.to_string()));
            }
        };

        log::debug!("Bytes read from server: {}", bytes_read);

        if bytes_read == 0 {
            return Ok(None);
        }

        // Decrypt in place (using aligned buffer slice)
        let pt_len = decrypt_in_place_buf(&mut bytes_in[ALIGN_PAD..])
            .map_err(|e| Status::new(tonic::Code::Internal, e.to_string()))?;

        // The plaintext is in bytes_in[ALIGN_PAD + PUBKEY_LEN + NONCE_LEN .. ]
        let start = ALIGN_PAD + PUBKEY_LEN + NONCE_LEN;
        let end = start + pt_len;

        let item = Message::decode(&bytes_in[start..end])
            .map(Option::Some)
            .map_err(from_decode_error)?;

        Ok(item)
    }
}

fn from_anyhow_error(error: anyhow::Error) -> Status {
    Status::new(tonic::Code::Internal, error.to_string())
}

fn from_decode_error(error: prost::DecodeError) -> Status {
    Status::new(tonic::Code::Internal, error.to_string())
}

pub fn encode_with_chacha<T, U>(msg: T) -> Result<Vec<u8>>
where
    T: Message + Send + 'static,
    U: Message + Default + Send + 'static,
{
    // Legacy function support
    // We can use a simpler approach or reuse the aligned logic if we care about perf here.
    // Since this is used in tests/benchmarks, let's use optimized aligned logic.

    let encoded_len = msg.encoded_len();
    let total_len = PUBKEY_LEN + NONCE_LEN + encoded_len + TAG_LEN;

    let mut out = Vec::with_capacity(ALIGN_PAD + total_len);
    out.extend_from_slice(&[0u8; ALIGN_PAD]);

    // Write header
    // Helper `encrypt_into_vec` doesn't support alignment/msg input easily.
    // Replicating logic similar to `encode_encrypt_into_buf` but to Vec.

    let server_public: PublicKey = PublicKey::from(SERVER_PUBKEY);
    let rng = rand_chacha::ChaCha20Rng::from_entropy();
    let client_secret = EphemeralSecret::random_from_rng(rng);
    let client_public = PublicKey::from(&client_secret);
    let shared_secret = client_secret.diffie_hellman(&server_public);
    add_key_history(*client_public.as_bytes(), *shared_secret.as_bytes());

    let cipher = chacha20poly1305::XChaCha20Poly1305::new(GenericArray::from_slice(
        shared_secret.as_bytes(),
    ));
    let nonce = chacha20poly1305::XChaCha20Poly1305::generate_nonce(&mut OsRng);

    out.extend_from_slice(client_public.as_bytes());
    out.extend_from_slice(nonce.as_slice());

    msg.encode(&mut out)?;

    let pt_start = ALIGN_PAD + PUBKEY_LEN + NONCE_LEN;
    let tag = cipher
        .encrypt_in_place_detached(&nonce, &[], &mut out[pt_start..])
        .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

    out.extend_from_slice(tag.as_slice());

    // Return buffer without padding
    Ok(out[ALIGN_PAD..].to_vec())
}

pub fn decode_with_chacha<T, U>(data: &[u8]) -> Result<U>
where
    T: Message + Send + 'static,
    U: Message + Default + Send + 'static,
{
    // Optimize with alignment
    let mut buf = Vec::with_capacity(ALIGN_PAD + data.len());
    buf.extend_from_slice(&[0u8; ALIGN_PAD]);
    buf.extend_from_slice(data);

    let pt_len = decrypt_in_place_buf(&mut buf[ALIGN_PAD..])?;

    let start = ALIGN_PAD + PUBKEY_LEN + NONCE_LEN;
    let end = start + pt_len;

    U::decode(&buf[start..end]).context("Failed to decode protobuf message")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_history_bounded() {
        let mut key_history = key_history().lock().unwrap();
        for i in 0..KEY_CACHE_SIZE * 2 {
            let mut pub_key = [0u8; 32];
            let mut shared_secret = [0u8; 32];
            pub_key[0] = i as u8;
            pub_key[1] = (i >> 8) as u8;
            shared_secret[0] = i as u8;
            shared_secret[1] = (i >> 8) as u8;
            key_history.put(pub_key, shared_secret);
        }
        assert_eq!(key_history.len(), KEY_CACHE_SIZE);
    }
}
