use anyhow::Result;
use pb::c2::{
    ClaimTasksRequest, ClaimTasksResponse, FetchAssetRequest, FetchAssetResponse,
    ReportCredentialRequest, ReportCredentialResponse, ReportFileRequest, ReportFileResponse,
    ReportProcessListRequest, ReportProcessListResponse, ReportTaskOutputRequest,
    ReportTaskOutputResponse,
};
use std::{
    future::Future,
    sync::mpsc::{Receiver, Sender},
};

pub trait Transport: Clone + Send {
    ///
    /// Contact the server for new tasks to execute.
    fn claim_tasks(
        &mut self,
        request: ClaimTasksRequest,
    ) -> impl Future<Output = Result<ClaimTasksResponse>> + Send;

    ///
    /// Fetch an asset from the server, returning one or more chunks of data.
    /// The maximum size of these chunks is determined by the server.
    /// The server should reply with two headers:
    ///   - "sha3-256-checksum": A SHA3-256 digest of the entire file contents.
    ///   - "file-size": The number of bytes contained by the file.
    ///
    /// If no associated file can be found, a NotFound status error is returned.
    fn fetch_asset(
        &mut self,
        request: FetchAssetRequest,
        sender: Sender<FetchAssetResponse>,
    ) -> impl Future<Output = Result<()>> + Send;

    ///
    /// Report a credential to the server.
    fn report_credential(
        &mut self,
        request: ReportCredentialRequest,
    ) -> impl Future<Output = Result<ReportCredentialResponse>> + Send;

    ///
    /// Report a file from the host to the server.
    /// Providing content of the file is optional. If content is provided:
    ///   - Hash will automatically be calculated and the provided hash will be ignored.
    ///   - Size will automatically be calculated and the provided size will be ignored.
    /// Content is provided as chunks, the size of which are up to the agent to define (based on memory constraints).
    /// Any existing files at the provided path for the host are replaced.
    fn report_file(
        &mut self,
        request: Receiver<ReportFileRequest>,
    ) -> impl Future<Output = Result<ReportFileResponse>> + Send;

    ///
    /// Report the active list of running processes. This list will replace any previously reported
    /// lists for the same host.
    fn report_process_list(
        &mut self,
        request: ReportProcessListRequest,
    ) -> impl Future<Output = Result<ReportProcessListResponse>> + Send;

    ///
    /// Report execution output for a task.
    fn report_task_output(
        &mut self,
        request: ReportTaskOutputRequest,
    ) -> impl Future<Output = Result<ReportTaskOutputResponse>> + Send;
}
