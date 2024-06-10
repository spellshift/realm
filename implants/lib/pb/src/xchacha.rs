use bytes::{Buf, BufMut};
use chacha20poly1305::{aead::generic_array::GenericArray, aead::Aead, AeadCore, KeyInit};
use prost::Message;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    marker::PhantomData,
};
use tonic::{
    codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder},
    Status,
};

/// A [`Codec`] that implements `application/grpc+json` via the serde library.
#[derive(Debug, Clone)]
pub struct ChachaCodec<T, U>(PhantomData<(T, U)>, ChaChaSvc);

impl<T, U> Default for ChachaCodec<T, U> {
    fn default() -> Self {
        log::debug!("CODEC LOADED");
        Self(
            PhantomData,
            ChaChaSvc {
                key: b"helloworld".to_vec(),
            },
        )
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
        ChachaEncrypt(
            PhantomData,
            ChaChaSvc {
                key: b"helloworld".to_vec(),
            },
        )
    }

    fn decoder(&mut self) -> Self::Decoder {
        ChachaDecrypt(
            PhantomData,
            ChaChaSvc {
                key: b"helloworld".to_vec(),
            },
        )
    }
}

// ---

#[derive(Debug, Clone)]
pub struct ChaChaSvc {
    key: Vec<u8>,
}

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
            println!("DANGER can't add to the buffer.");
        }

        let key = self.1.key.as_slice();
        let mut hasher = Sha256::new();
        hasher.update(key);
        let key_hash = hasher.finalize();

        let cipher = chacha20poly1305::XChaCha20Poly1305::new(GenericArray::from_slice(&key_hash));
        let nonce = chacha20poly1305::XChaCha20Poly1305::generate_nonce(&mut OsRng);

        let pt_vec = item.encode_to_vec();

        println!("ENCODE DEC: {:?}", pt_vec);
        let _ = buf.writer().write_all(&pt_vec);

        let ciphertext = match cipher.encrypt(&nonce, pt_vec.as_slice()) {
            Ok(ct) => ct,
            Err(err) => {
                println!("err: {:?}", err);
                return Err(Status::new(tonic::Code::Internal, err.to_string()));
            }
        };

        let _ = buf.writer().write_all(nonce.as_slice());
        buf.writer().write_all(ciphertext.as_slice())?;

        println!("ENCODE ENC: {:?}", buf);

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
        // if !buf.has_remaining() {
        //     return Err(Status::new(
        //         tonic::Code::Internal,
        //         "Unable to allocate new buffer space",
        //     ));
        // }

        let mut reader = buf.reader();
        let mut bytes_in = vec![0; DEFAULT_CODEC_BUFFER_SIZE];
        let bytes_read = match reader.read(&mut bytes_in) {
            Ok(n) => n,
            Err(err) => {
                println!("err: {:?}", err);
                return Err(Status::new(tonic::Code::Internal, err.to_string()));
            }
        };

        let ciphertext = &bytes_in[0..bytes_read];
        let nonce = &ciphertext[0..24];
        let ciphertext = &ciphertext[24..];

        println!("DECODE ENC: {:?}", ciphertext);

        let key = self.1.key.as_slice();
        let mut hasher = Sha256::new();
        hasher.update(key);
        let key_hash = hasher.finalize();
        let cipher = chacha20poly1305::XChaCha20Poly1305::new(GenericArray::from_slice(&key_hash));

        let plaintext = match cipher.decrypt(GenericArray::from_slice(nonce), ciphertext.as_ref()) {
            Ok(pt) => pt,
            Err(err) => {
                println!("err: {:?}", err);
                return Err(Status::new(tonic::Code::Internal, err.to_string()));
            }
        };

        println!("DECODE DEC: {:?}", plaintext);

        let item = Message::decode(bytes::Bytes::from(plaintext))
            .map(Option::Some)
            .map_err(from_decode_error)?;

        // let plaintext = bytes::Bytes::copy_from_slice(ciphertext);
        // let item: Option<U> = Message::decode(plaintext)
        //     .map(Option::Some)
        //     .map_err(from_decode_error)?;

        Ok(item)
    }
}

fn from_decode_error(error: prost::DecodeError) -> Status {
    // Map Protobuf parse errors to an INTERNAL status code, as per
    // https://github.com/grpc/grpc/blob/master/doc/statuscodes.md
    Status::new(tonic::Code::Internal, error.to_string())
}

// ---
