use anyhow::{Context, Result};
use bytes::{Buf, BufMut};
use chacha20poly1305::{aead::generic_array::GenericArray, aead::Aead, AeadCore, KeyInit};
use lru::LruCache;
use prost::Message;
use rand::rngs::OsRng;
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

/* Default server pubkey used for testing when IMIX_SERVER_PUBKEY is not provided.
 * To find the server's pubkey check the startup messages on the server look for `[INFO] Public key: <SERVER_PUBKEY>`
 */
const DEFAULT_SERVER_PUBKEY: [u8; 32] = [
    165, 30, 122, 188, 50, 89, 111, 214, 247, 4, 189, 217, 188, 37, 200, 190, 2, 180, 175, 107,
    194, 147, 177, 98, 103, 84, 99, 120, 72, 73, 87, 37,
];
// ------------

const KEY_CACHE_SIZE: usize = 1024;
const PUBKEY_LEN: usize = 32;
const NONCE_LEN: usize = 24;

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

fn encrypt_impl(pt_vec: Vec<u8>, server_pubkey: [u8; 32]) -> Result<Vec<u8>> {
    // Store server pubkey
    let server_public: PublicKey = PublicKey::from(server_pubkey);

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

    // Encrypt data
    let ciphertext: Vec<u8> = match cipher.encrypt(&nonce, pt_vec.as_slice()) {
        Ok(ct) => ct,
        Err(err) => {
            #[cfg(debug_assertions)]
            log::debug!(
                "encode error unable to read bytes while encrypting: {:?}",
                err
            );
            return Err(anyhow::anyhow!("Encryption failed: {:?}", err));
        }
    };

    // Build result: pubkey + nonce + ciphertext
    let mut result = Vec::new();
    result.extend_from_slice(client_public.as_bytes());
    result.extend_from_slice(nonce.as_slice());
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

fn decrypt_impl(ct_vec: Vec<u8>) -> Result<Vec<u8>> {
    if ct_vec.is_empty() {
        return Ok(vec![]);
    }

    if ct_vec.len() < PUBKEY_LEN + NONCE_LEN {
        anyhow::bail!("Message too small to contain public key and nonce");
    }

    // Extract components
    let client_public = ct_vec
        .get(0..PUBKEY_LEN)
        .context("Input buffer doesn't have enough bytes for public key")?;

    let nonce = ct_vec
        .get(PUBKEY_LEN..PUBKEY_LEN + NONCE_LEN)
        .context("Input buffer doesn't have enough bytes for nonce")?;

    let ciphertext = ct_vec
        .get(PUBKEY_LEN + NONCE_LEN..)
        .context("Input buffer doesn't have enough bytes for ciphertext")?;

    // Get private key based on messages public key

    let client_private_bytes = get_key(client_public.try_into()?).map_err(from_anyhow_error)?;

    let cipher =
        chacha20poly1305::XChaCha20Poly1305::new(GenericArray::from_slice(&client_private_bytes));

    let plaintext = cipher
        .decrypt(GenericArray::from_slice(nonce), ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;

    Ok(plaintext)
}

// ------------

#[derive(Debug, Clone)]
pub struct ChaChaSvc {
    server_pubkey: [u8; 32],
}

impl Default for ChaChaSvc {
    fn default() -> Self {
        Self {
            server_pubkey: DEFAULT_SERVER_PUBKEY,
        }
    }
}

impl ChaChaSvc {
    pub fn new(server_pubkey: [u8; 32]) -> Self {
        Self { server_pubkey }
    }
}

#[derive(Debug, Clone)]
pub struct ChachaCodec<T, U>(PhantomData<(T, U)>, ChaChaSvc);

impl<T, U> Default for ChachaCodec<T, U> {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        log::debug!("Loaded custom codec with xchacha encryption (using default pubkey)");
        Self(PhantomData, ChaChaSvc::default())
    }
}

impl<T, U> ChachaCodec<T, U> {
    pub fn new(server_pubkey: [u8; 32]) -> Self {
        #[cfg(debug_assertions)]
        log::debug!("Loaded custom codec with xchacha encryption (using provided pubkey)");
        Self(PhantomData, ChaChaSvc::new(server_pubkey))
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
        ChachaEncrypt(PhantomData, self.1.clone())
    }

    fn decoder(&mut self) -> Self::Decoder {
        ChachaDecrypt(PhantomData, self.1.clone())
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
            // This should never happen but if it does the agent will be unable to queue new messages to the buffer until it's drained.
            #[cfg(debug_assertions)]
            log::debug!("DANGER can't add to the buffer.");
        }

        let _ = match encrypt_impl(item.encode_to_vec(), self.1.server_pubkey) {
            Ok(pub_nonce_ct) => buf.writer().write_all(&pub_nonce_ct),
            Err(err) => return Err(Status::new(tonic::Code::Internal, err.to_string())),
        };

        Ok(())
    }
}

// ---
//

#[derive(Debug)]
#[allow(dead_code)]
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
        let mut reader = buf.reader();
        let mut bytes_in = Vec::new();
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
            let item = Message::decode(bytes_in.get(0..bytes_read).unwrap())
                .map(Option::Some)
                .map_err(from_decode_error)?;

            return Ok(item);
        }

        let plaintext = decrypt_impl(bytes_in.get(0..bytes_read).unwrap().to_vec())
            .map_err(from_anyhow_error)?;

        // Serialize
        let item = Message::decode(bytes::Bytes::from(plaintext))
            .map(Option::Some)
            .map_err(from_decode_error)?;

        Ok(item)
    }
}

fn from_anyhow_error(error: anyhow::Error) -> Status {
    Status::new(tonic::Code::Internal, error.to_string())
}

fn from_decode_error(error: prost::DecodeError) -> Status {
    // Map Protobuf parse errors to an INTERNAL status code, as per
    // https://github.com/grpc/grpc/blob/master/doc/statuscodes.md
    Status::new(tonic::Code::Internal, error.to_string())
}

pub fn encode_with_chacha<T, U>(msg: T, server_pubkey: [u8; 32]) -> Result<Vec<u8>>
where
    T: Message + Send + 'static,
    U: Message + Default + Send + 'static,
{
    encrypt_impl(msg.encode_to_vec(), server_pubkey)
}

pub fn decode_with_chacha<T, U>(data: &[u8]) -> Result<U>
where
    T: Message + Send + 'static,
    U: Message + Default + Send + 'static,
{
    // Decode protobuf
    match decrypt_impl(data.to_vec()) {
        Ok(plaintext) => {
            U::decode(bytes::Bytes::from(plaintext)).context("Failed to decode protobuf message")
        }
        Err(err) => Err(err),
    }
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
