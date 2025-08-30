use anyhow::{Context, Result};
use bytes::{Buf, BufMut};
use chacha20poly1305::{aead::generic_array::GenericArray, aead::Aead, AeadCore, KeyInit};
use const_decoder::Decoder as const_decode;
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

/* Compile-time constant for the server pubkey, derived from the IMIX_SERVER_PUBKEY environment variable during compilation.
 * To find the servers pubkey check the startup messages on the server look for `[INFO] Public key: <SERVER_PUBKEY>`
 */
static SERVER_PUBKEY: [u8; 32] = const_decode::Base64.decode(env!("IMIX_SERVER_PUBKEY").as_bytes());

// ------------

const KEY_CACHE_SIZE: usize = 1024;

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

fn del_key(pub_key: [u8; 32]) -> Option<[u8; 32]> {
    key_history().lock().unwrap().pop(&pub_key)
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
            // This should never happen but if it does the agent will be unable to queue new messages to the buffer until it's drained.
            #[cfg(debug_assertions)]
            log::debug!("DANGER can't add to the buffer.");
        }

        // Store server pubkey
        let server_public = PublicKey::from(SERVER_PUBKEY);

        // Generate ephemeral keys
        let rng = rand_chacha::ChaCha20Rng::from_entropy();
        let client_secret = EphemeralSecret::random_from_rng(rng);
        let client_public = PublicKey::from(&client_secret);

        // Generate shared secret
        let shared_secret = client_secret.diffie_hellman(&server_public);
        add_key_history(*client_public.as_bytes(), *shared_secret.as_bytes());

        // Generate nonce
        let cipher = chacha20poly1305::XChaCha20Poly1305::new(GenericArray::from_slice(
            shared_secret.as_bytes(),
        ));
        let nonce = chacha20poly1305::XChaCha20Poly1305::generate_nonce(&mut OsRng);

        // Encrypt data
        let pt_vec = item.encode_to_vec();
        let ciphertext = match cipher.encrypt(&nonce, pt_vec.as_slice()) {
            Ok(ct) => ct,
            Err(err) => {
                #[cfg(debug_assertions)]
                log::debug!(
                    "encode error unable to read bytes while encrypting: {:?}",
                    err
                );
                return Err(Status::new(tonic::Code::Internal, err.to_string()));
            }
        };

        // Write pubkey + nonce + cipher text
        buf.writer().write_all(client_public.as_bytes())?;
        buf.writer().write_all(nonce.as_slice())?;
        buf.writer().write_all(ciphertext.as_slice())?;

        Ok(())
    }
}

// ---
//
const DEFAULT_CODEC_BUFFER_SIZE: usize = 8 * 1024;
const PUBKEY_LEN: usize = 32;
const NONCE_LEN: usize = 24;

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
        let mut reader = buf.reader();
        let mut bytes_in = vec![0; DEFAULT_CODEC_BUFFER_SIZE];
        let bytes_read = match reader.read(&mut bytes_in) {
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

        if bytes_read < PUBKEY_LEN + NONCE_LEN {
            let err =
                anyhow::anyhow!("Message from server is too small to contain public key and nonce");
            log::debug!(
                "Input buffer from server during decode faild validation: {:?}",
                err
            );
            return Err(Status::new(tonic::Code::Internal, err.to_string()));
        }
        let buf = bytes_in
            .get(0..bytes_read)
            .context("Bytes read doesn't match buffer size")
            .map_err(from_anyhow_error)?;

        let client_public = buf
            .get(0..PUBKEY_LEN)
            .context("Input buffer doesn't have enough bytes for public key")
            .map_err(from_anyhow_error)?;

        let nonce = buf
            .get(PUBKEY_LEN..PUBKEY_LEN + NONCE_LEN)
            .context("Input buffer doesn't have enough bytes for nonce")
            .map_err(from_anyhow_error)?;

        let ciphertext = buf
            .get(PUBKEY_LEN + NONCE_LEN..)
            .context("Input buffer doesn't have enough bytes for ciphertext")
            .map_err(from_anyhow_error)?;

        // Get private key based on messages public key
        let tmp_client_public_bytes = client_public.to_vec();
        let client_public_bytes = match tmp_client_public_bytes.try_into() {
            Ok(arr) => arr,
            Err(err) => {
                #[cfg(debug_assertions)]
                log::debug!("Unable to cast public key bytes from vector to slice during decode function: {:?}", err);

                return Err(Status::new(
                    tonic::Code::Internal,
                    "Unable to cast public key bytes from vector to slice during decode function",
                ));
            }
        };

        let client_private_bytes = get_key(client_public_bytes).map_err(from_anyhow_error)?;
        // Shouldn't need private key again once the message has been decrypted
        del_key(client_public_bytes);

        let cipher = chacha20poly1305::XChaCha20Poly1305::new(GenericArray::from_slice(
            &client_private_bytes,
        ));

        // Decrypt message
        let plaintext = match cipher.decrypt(GenericArray::from_slice(nonce), ciphertext.as_ref()) {
            Ok(pt) => pt,
            Err(err) => {
                #[cfg(debug_assertions)]
                log::debug!("Error decrypting response: {:?}", err);
                return Err(Status::new(tonic::Code::Internal, err.to_string()));
            }
        };

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
