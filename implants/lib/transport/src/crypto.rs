use anyhow::Result;
use bytes::Bytes;
use tonic::transport::Channel;

pub trait CryptoSvc {
    fn new(inner: Channel) -> Self;
    fn encrypt(bytes: Bytes) -> Result<Bytes>;
    fn decrypt(bytes: Bytes) -> Result<Bytes>;
}

// Is there a way to require things implementing
// Crypto service implement tower::Service?
