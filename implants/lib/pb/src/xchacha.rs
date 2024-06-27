use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use bytes::{Buf, BufMut};
use chacha20poly1305::{aead::generic_array::GenericArray, aead::Aead, AeadCore, KeyInit};
use prost::Message;
use rand::rngs::OsRng;
use rand_chacha::rand_core::SeedableRng;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    hash::Hash,
    io::{Read, Write},
    marker::PhantomData,
    sync::{Arc, Mutex, OnceLock, RwLock},
};
use tonic::{
    codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder},
    Status,
};
use x25519_dalek::{EphemeralSecret, PublicKey};

#[derive(Debug, Clone)]
pub struct ChaChaSvc {
    key: Vec<u8>,
    priv_key_history: HashMap<[u8; 32], [u8; 32]>,
}

fn key_history() -> &'static Mutex<HashMap<[u8; 32], [u8; 32]>> {
    static ARRAY: OnceLock<Mutex<HashMap<[u8; 32], [u8; 32]>>> = OnceLock::new();
    ARRAY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn add_key_history(pub_key: [u8; 32], shared_secret: [u8; 32]) {
    key_history().lock().unwrap().insert(pub_key, shared_secret);
}

fn get_key(pub_key: [u8; 32]) -> [u8; 32] {
    *key_history().lock().unwrap().get(&pub_key).unwrap()
}

// !!!! DANGER !!!!!
// This is the default static key set to ensure compatability between server and client
// in debug builds. This key must be changed in order to protect data. We're still in the
// process of implementing app layer crypto and this is temporary. Do not rely on the
// app layer crypto without changing this default.
const IMIX_ENCRYPT_KEY_DEFAULT: &str = "I Don't care how small the room is I cast fireball";
const IMIX_ENCRYPT_KEY: Option<&'static str> = option_env!("IMIX_ENCRYPT_KEY",);

const SERVER_PUB_KEY: [u8; 32] = [
    115, 191, 95, 40, 160, 88, 16, 88, 37, 61, 11, 49, 240, 4, 125, 171, 127, 117, 245, 6, 242,
    215, 197, 247, 122, 187, 197, 169, 155, 249, 205, 61,
];

impl Default for ChaChaSvc {
    fn default() -> Self {
        Self {
            key: match IMIX_ENCRYPT_KEY {
                Some(res) => res.into(),
                None => IMIX_ENCRYPT_KEY_DEFAULT.into(),
            },
            priv_key_history: HashMap::new(),
        }
    }
}

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
            // Can't add to the buffer.
            #[cfg(debug_assertions)]
            log::debug!("DANGER can't add to the buffer.");
        }

        // Store server pubkey
        let server_public_bytes = SERVER_PUB_KEY;
        let server_public = PublicKey::from(server_public_bytes);

        // Generate ephemeral keys
        let rng = rand_chacha::ChaCha20Rng::from_entropy();
        let client_secret = EphemeralSecret::random_from_rng(rng);
        let client_public = PublicKey::from(&client_secret);
        log::debug!("client_public: {:?}", client_public);

        // Generate shared secret
        let shared_secret = client_secret.diffie_hellman(&server_public);
        add_key_history(*client_public.as_bytes(), *shared_secret.as_bytes());

        // let key = shared_secret.as_bytes();
        // let mut hasher = Sha256::new();
        // hasher.update(IMIX_ENCRYPT_KEY_DEFAULT);
        // let key = hasher.finalize();
        // log::debug!("[DEBUG] {:?}", key);
        log::debug!("[DEBUG] derived key {:?}", shared_secret.as_bytes());

        let cipher = chacha20poly1305::XChaCha20Poly1305::new(GenericArray::from_slice(
            shared_secret.as_bytes(),
        ));
        let nonce = chacha20poly1305::XChaCha20Poly1305::generate_nonce(&mut OsRng);

        let pt_vec = item.encode_to_vec();

        let ciphertext = match cipher.encrypt(&nonce, pt_vec.as_slice()) {
            Ok(ct) => ct,
            Err(err) => {
                #[cfg(debug_assertions)]
                log::debug!("err: {:?}", err);
                return Err(Status::new(tonic::Code::Internal, err.to_string()));
            }
        };

        let _ = buf.writer().write_all(client_public.as_bytes());
        let _ = buf.writer().write_all(nonce.as_slice());
        buf.writer().write_all(ciphertext.as_slice())?;

        Ok(())
    }
}

// ---
//
const DEFAULT_CODEC_BUFFER_SIZE: usize = 8 * 1024;

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
        let mut reader = buf.reader();
        let mut bytes_in = vec![0; DEFAULT_CODEC_BUFFER_SIZE];
        let bytes_read = match reader.read(&mut bytes_in) {
            Ok(n) => n,
            Err(err) => {
                #[cfg(debug_assertions)]
                log::debug!("err: {:?}", err);
                return Err(Status::new(tonic::Code::Internal, err.to_string()));
            }
        };

        // TODO validate buffer size to avoid index out of bounds accesses
        let buf = bytes_in.get(0..bytes_read).unwrap();
        let msg_pub_key_ref = buf.get(0..32).unwrap();
        let nonce = buf.get(32..56).unwrap();
        let ciphertext = buf.get(56..).unwrap();

        log::debug!("msg_pub_key: {:?}", buf);
        let msg_pub_key: [u8; 32] = msg_pub_key_ref.to_vec().try_into().unwrap();

        let key = get_key(msg_pub_key);
        log::debug!("priv_key_history: {:?}", key);
        let cipher = chacha20poly1305::XChaCha20Poly1305::new(GenericArray::from_slice(&key));

        let plaintext = match cipher.decrypt(GenericArray::from_slice(nonce), ciphertext.as_ref()) {
            Ok(pt) => pt,
            Err(err) => {
                #[cfg(debug_assertions)]
                log::debug!("err: {:?}", err);
                return Err(Status::new(tonic::Code::Internal, err.to_string()));
            }
        };

        let item = Message::decode(bytes::Bytes::from(plaintext))
            .map(Option::Some)
            .map_err(from_decode_error)?;

        Ok(item)
    }
}

fn from_decode_error(error: prost::DecodeError) -> Status {
    // Map Protobuf parse errors to an INTERNAL status code, as per
    // https://github.com/grpc/grpc/blob/master/doc/statuscodes.md
    Status::new(tonic::Code::Internal, error.to_string())
}

pub fn decode_b64(content: String) -> Result<Vec<u8>> {
    Ok(general_purpose::STANDARD.decode(content.as_bytes())?)
}
