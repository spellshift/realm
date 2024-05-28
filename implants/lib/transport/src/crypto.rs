use anyhow::Result;
use bytes::Bytes;
use tonic::transport::Channel;

pub trait CryptoSvc {
    fn new(inner: Channel, key: Vec<u8>) -> Self;
    fn encrypt(&self, bytes: Bytes) -> Bytes;
    fn decrypt(&self, bytes: Bytes) -> Bytes;
}

// Is there a way to require things implementing
// Crypto service implement tower::Service?
