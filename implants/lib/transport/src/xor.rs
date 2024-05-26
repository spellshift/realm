use crate::crypto::CryptoSvc;
use anyhow::Result;
use bytes::Bytes;
use hyper::body::HttpBody as HyperHttpBody;
use hyper::http::{Request, Response};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::StreamExt;
use tonic::body::BoxBody;
use tonic::transport::Body;
use tonic::transport::Channel;
use tower::Service;

#[derive(Debug, Clone)]
pub struct XorSvc {
    inner: Channel,
}

const XOR_KEY: &[u8; 5] = b"\x01\x02\x03\x04\x05";

impl CryptoSvc for XorSvc {
    fn new(inner: Channel) -> Self {
        XorSvc { inner }
    }
    fn decrypt(bytes: Bytes) -> Bytes {
        let mut res_bytes = Vec::new();
        for (i, x) in bytes.into_iter().enumerate() {
            res_bytes.push(x ^ XOR_KEY[i % XOR_KEY.len()]);
        }

        bytes::Bytes::from(res_bytes)
    }
    fn encrypt(bytes: Bytes) -> Bytes {
        let mut res_bytes = Vec::new();
        for (i, x) in bytes.into_iter().enumerate() {
            res_bytes.push(x ^ XOR_KEY[i % XOR_KEY.len()]);
        }

        bytes::Bytes::from(res_bytes)
    }
}

// Should this move into the `crytosvc` trait
// Would be nice to have all this logic handled by us
// Then crypto impls are just an encrypt and decrypt.
impl Service<Request<BoxBody>> for XorSvc {
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

        Box::pin(async move {
            // Encrypt request
            let new_req = {
                let (parts, body) = req.into_parts();
                let new_body = body.map_data(crate::xor::XorSvc::encrypt).boxed_unsync();

                Request::from_parts(parts, new_body)
            };

            let response: Response<Body> = inner.call(new_req).await?;

            // Decrypt response
            let new_resp = {
                let (parts, mut body) = response.into_parts();

                let body_bytes = body.data().await.unwrap().unwrap();
                let dec_body_bytes = crate::xor::XorSvc::decrypt(body_bytes);
                let new_body = hyper::body::Body::from(dec_body_bytes);

                Response::from_parts(parts, new_body)
            };

            Ok(new_resp)
        })
    }
}
