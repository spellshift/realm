use bytes::{Buf, Bytes};
use chacha20poly1305::{
    aead::{Aead, OsRng},
    AeadCore, XChaCha20Poly1305,
};
use prost::Message;
use sha2::{digest::generic_array::GenericArray, Digest as _, Sha256};
use std::marker::PhantomData;
use tonic::{
    codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder},
    transport::Channel,
    Status,
};

use crate::crypto::CryptoSvc;

#[derive(Debug, Clone)]
pub struct ChaChaSvc {
    inner: Channel,
    key: Vec<u8>,
}

impl CryptoSvc for ChaChaSvc {
    fn new(inner: Channel, key: Vec<u8>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(key);
        let padded_key = hasher.finalize().to_vec();

        ChaChaSvc {
            inner,
            key: padded_key,
        }
    }

    fn decrypt(&self, bytes: Bytes) -> Bytes {
        log::debug!("enc bytes: {:?}", bytes.to_vec());

        let nonce = &bytes.clone()[..24];
        let ciphertext = &bytes.clone()[24..];
        let cipher =
            <XChaCha20Poly1305 as chacha20poly1305::KeyInit>::new_from_slice(&self.key).unwrap();

        let res_bytes = match cipher.decrypt(GenericArray::from_slice(nonce), ciphertext) {
            Ok(res) => res,
            Err(err) => {
                #[cfg(debug_assertions)]
                log::error!("Decrypt failed: {}", err.to_string());

                Vec::new()
            }
        };

        {
            log::debug!("dec bytes: {:?}", res_bytes.to_vec());
        }

        bytes::Bytes::from_iter(res_bytes)
    }

    fn encrypt(&self, bytes: Bytes) -> Bytes {
        let cipher =
            <XChaCha20Poly1305 as chacha20poly1305::KeyInit>::new_from_slice(self.key.as_slice())
                .unwrap();
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng); // 192-bits; unique per message

        let mesg = match cipher.encrypt(&nonce, bytes.as_ref()) {
            Ok(mesg) => mesg,
            Err(err) => {
                #[cfg(debug_assertions)]
                log::error!("Encrypt failed: {:?}", err.to_string());

                Vec::new()
            }
        };

        let res = [nonce.to_vec(), mesg].concat();

        bytes::Bytes::from(res)
    }
}

#[derive(Debug)]
pub struct ChaChaEncoder<T>(PhantomData<T>);

impl<T> Encoder for ChaChaEncoder<T> {
    type Item = T;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, buf: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        log::debug!("Encode buf: {:?}", buf);
        // serde_json::to_writer(buf.writer(), &item).map_err(|e| Status::internal(e.to_string()))
        todo!();
    }
}

#[derive(Debug)]
pub struct ChaChaDecoder<U>(PhantomData<U>);

impl<U> Decoder for ChaChaDecoder<U> {
    type Item = U;
    type Error = Status;

    fn decode(&mut self, buf: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        if !buf.has_remaining() {
            return Ok(None);
        }

        log::debug!("Decode: {:?}", buf);

        // let item: Self::Item =
        //     serde_json::from_reader(buf.reader()).map_err(|e| Status::internal(e.to_string()))?;
        // let item: Self::Item =
        // Ok(Some(item))
        todo!();
    }
}

#[derive(Debug, Clone)]
pub struct ChaChaCodec<T, U>(PhantomData<(T, U)>);

impl<T, U> Default for ChaChaCodec<T, U> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T, U> Codec for ChaChaCodec<T, U>
where
    T: Message + Send + 'static,
    U: Message + Default + Send + 'static,
{
    type Encode = T;
    type Decode = U;
    type Encoder = ChaChaEncoder<T>;
    type Decoder = ChaChaDecoder<U>;

    fn encoder(&mut self) -> Self::Encoder {
        ChaChaEncoder(PhantomData)
    }

    fn decoder(&mut self) -> Self::Decoder {
        ChaChaDecoder(PhantomData)
    }
}
