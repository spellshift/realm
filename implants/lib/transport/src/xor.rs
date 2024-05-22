use crate::crypto::CryptoSvc;
use anyhow::Result;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::body::HttpBody as HyperHttpBody;
use hyper::http::{Request, Response};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::StreamExt;
use tonic::body::BoxBody;
use tonic::transport::Body;
use tonic::transport::Channel;
use tonic::IntoStreamingRequest;
use tower::Service;

#[derive(Debug, Clone)]
pub struct XorSvc {
    inner: Channel,
}

impl CryptoSvc for XorSvc {
    fn new(inner: Channel) -> Self {
        XorSvc { inner }
    }
    fn decrypt(bytes: Bytes) -> Result<Bytes> {
        Ok(bytes)
    }
    fn encrypt(bytes: Bytes) -> Result<Bytes> {
        Ok(bytes)
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
                // This in theory doesn't await
                // TODO: Validate that theory by making sure
                // it doesn't hold all of a large request in memory
                let body = body
                    .map_data(move |x| {
                        let enc_bytes = x.into_iter().map(|x| x + 0).collect::<bytes::Bytes>();
                        #[cfg(debug_assertions)]
                        log::debug!("Request bytes: {:?}", enc_bytes);
                        enc_bytes
                    })
                    .boxed_unsync();

                Request::from_parts(parts, body)
            };

            let response: Response<Body> = inner.call(new_req).await?;

            // Decrypt response
            let (parts, body) = response.into_parts();

            // TODO: Why doesn't this hit?
            // Do i need to use a different map funciton?
            let mut new_body: Body = body
                .map_data(move |x| {
                    let enc_bytes = x.into_iter().map(|x| x + 1).collect::<bytes::Bytes>();
                    #[cfg(debug_assertions)]
                    log::debug!("Response bytes: {:?}", enc_bytes);
                    enc_bytes
                })
                .into_inner();

            {
                let tmp = new_body.data().await.unwrap().unwrap();
                log::debug!("HERE: {:?}", tmp);
            }

            let new_resp = Response::from_parts(parts, new_body);

            Ok(new_resp)
        })
    }
}
