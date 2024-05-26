use crate::crypto::CryptoSvc;
use anyhow::Result;
use bytes::Bytes;
use hyper::body::HttpBody as HyperHttpBody;
use hyper::http::{Request, Response};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tonic::body::BoxBody;
use tonic::transport::Body;
use tonic::transport::Channel;
use tower::Service;

#[derive(Debug, Clone)]
pub struct XorSvc {
    inner: Channel,
}

impl CryptoSvc for XorSvc {
    fn new(inner: Channel) -> Self {
        XorSvc { inner }
    }
    fn decrypt(bytes: Bytes) -> Bytes {
        log::debug!("Encrypted response: {:?}", bytes);
        let res = bytes
            .into_iter()
            .map(|x| x ^ 0x69)
            .collect::<bytes::Bytes>();
        log::debug!("Decrypted response: {:?}", res);
        res
    }
    fn encrypt(bytes: Bytes) -> Bytes {
        log::debug!("Decrypted request: {:?}", bytes);
        let res = bytes
            .into_iter()
            .map(|x| x ^ 0x69)
            .collect::<bytes::Bytes>();
        log::debug!("Encrypted request: {:?}", res);
        res
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
                let body = body.map_data(crate::xor::XorSvc::encrypt).boxed_unsync();

                Request::from_parts(parts, body)
            };

            let response: Response<Body> = inner.call(new_req).await?;

            // Decrypt response
            let (parts, body) = response.into_parts();

            let mut new_body = body
                .map_data(|b| b.into_iter().map(|x| x ^ 0x69).collect::<bytes::Bytes>())
                .into_inner();

            log::debug!("new_body: {:?}", new_body.data().await.unwrap().unwrap());

            let new_resp = Response::from_parts(parts, new_body);

            Ok(new_resp)
        })
    }
}
