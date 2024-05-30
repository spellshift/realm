use crate::crypto::CryptoSvc;
use anyhow::Result;
use bytes::Bytes;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    KeySizeUser, XChaCha20Poly1305,
};
use hyper::body::HttpBody as HyperHttpBody;
use hyper::http::{Request, Response};
use sha2::{digest::generic_array::GenericArray, Digest, Sha256};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tonic::body::BoxBody;
use tonic::transport::Body;
use tonic::transport::Channel;
use tower::Service;

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
        if bytes.len() < 24 {
            #[cfg(debug_assertions)]
            log::error!("Message size smaller than nonce ({})", bytes.len());

            return bytes::Bytes::new();
        }

        let cipher = XChaCha20Poly1305::new_from_slice(self.key.as_slice()).unwrap();

        let nonce = &bytes.clone()[..24];
        let ciphertext = &bytes.clone()[24..];

        log::debug!("NONE: {:?}", nonce);
        log::debug!("KEY: {:?}", self.key);
        log::debug!("bytes: {:?}", bytes.to_vec());
        log::debug!("CT: {:?}", ciphertext.to_vec());

        let res_bytes = match cipher.decrypt(GenericArray::from_slice(nonce), ciphertext) {
            Ok(res) => res,
            Err(err) => {
                #[cfg(debug_assertions)]
                log::error!("Decrypt failed: {}", err.to_string());

                Vec::new()
            }
        };

        bytes::Bytes::from(res_bytes)
    }

    fn encrypt(&self, bytes: Bytes) -> Bytes {
        let cipher = XChaCha20Poly1305::new_from_slice(self.key.as_slice()).unwrap();
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

// Should this move into the `crytosvc` trait
// Would be nice to have all this logic handled by us
// Then crypto impls are just an encrypt and decrypt.
impl Service<Request<BoxBody>> for ChaChaSvc {
    type Response = Response<Body>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<BoxBody>) -> Self::Future {
        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        let clone: Channel = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        // This is so stupid but i'm not sure how to satisfying the borrowing.
        let self_clone = self.clone();
        let self_clone2 = self.clone();

        Box::pin(async move {
            // Encrypt request
            let new_req = {
                let (parts, body) = req.into_parts();
                let new_body = body.map_data(move |x| self_clone.encrypt(x)).boxed_unsync();

                Request::from_parts(parts, new_body)
            };

            let response: Response<Body> = inner.call(new_req).await?;

            // Decrypt response
            let new_resp = {
                let (parts, mut body) = response.into_parts();

                let body_bytes_tmp = body.data().await.unwrap_or(Ok(bytes::Bytes::new()));
                let body_bytes = body_bytes_tmp.unwrap_or(bytes::Bytes::new());
                let dec_body_bytes = self_clone2.decrypt(body_bytes);
                let new_body = hyper::body::Body::from(dec_body_bytes);

                Response::from_parts(parts, new_body)
            };

            Ok(new_resp)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chacha_enc_dec() -> anyhow::Result<()> {
        let endpoint = tonic::transport::Endpoint::from_static("127.0.0.1");
        let channel = endpoint.connect_lazy();

        let test_bytes = bytes::Bytes::from_static(
            "My super secret data that needs to be kept secret!".as_bytes(),
        );
        let chacha = ChaChaSvc::new(channel, "$up3r-S3cretPassword123!".as_bytes().to_vec());
        let enc = chacha.encrypt(test_bytes.clone());
        assert_ne!(enc, test_bytes.clone());
        let dec = chacha.decrypt(enc);
        assert_eq!(dec, test_bytes);
        Ok(())
    }

    #[tokio::test]
    async fn test_chacha_empty_enc_dec() -> anyhow::Result<()> {
        let endpoint = tonic::transport::Endpoint::from_static("127.0.0.1");
        let channel = endpoint.connect_lazy();

        let test_bytes = bytes::Bytes::new();
        let chacha = ChaChaSvc::new(channel, "$up3r-S3cretPassword123!".as_bytes().to_vec());
        let enc = chacha.encrypt(test_bytes.clone());
        assert_ne!(enc, test_bytes.clone());
        let dec = chacha.decrypt(enc);
        assert_eq!(dec, test_bytes);
        Ok(())
    }
}
