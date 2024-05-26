use anyhow::Result;
use bytes::Bytes;
use tonic::transport::Channel;

pub trait CryptoSvc {
    fn new(inner: Channel) -> Self;
    fn encrypt(bytes: Bytes) -> Bytes;
    fn decrypt(bytes: Bytes) -> Bytes;
}

// Is there a way to require things implementing
// Crypto service implement tower::Service?
