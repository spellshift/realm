use anyhow::Result;
use async_trait::async_trait;
use std::sync::mpsc::{Receiver, Sender};

#[async_trait]
pub trait TavernClient<T> {
    #[allow(clippy::new_ret_no_self)]
    async fn new(callback: String) -> Result<T>;
    ///
    /// Contact the server for new tasks to execute.
    async fn claim_tasks(
        &mut self,
        request: crate::pb::ClaimTasksRequest,
    ) -> Result<crate::pb::ClaimTasksResponse>;

    ///
    /// Download a file from the server, returning one or more chunks of data.
    /// The maximum size of these chunks is determined by the server.
    /// The server should reply with two headers:
    ///   - "sha3-256-checksum": A SHA3-256 digest of the entire file contents.
    ///   - "file-size": The number of bytes contained by the file.
    ///
    /// If no associated file can be found, a NotFound status error is returned.
    async fn download_file(
        &mut self,
        request: crate::pb::DownloadFileRequest,
    ) -> Result<Receiver<crate::pb::DownloadFileResponse>>;

    ///
    /// Report a file from the host to the server.
    /// Providing content of the file is optional. If content is provided:
    ///   - Hash will automatically be calculated and the provided hash will be ignored.
    ///   - Size will automatically be calculated and the provided size will be ignored.
    /// Content is provided as chunks, the size of which are up to the agent to define (based on memory constraints).
    /// Any existing files at the provided path for the host are replaced.
    async fn report_file(
        &mut self,
        request: Sender<crate::pb::ReportFileRequest>,
    ) -> Result<crate::pb::ReportFileResponse>;

    ///
    /// Report the active list of running processes. This list will replace any previously reported
    /// lists for the same host.
    async fn report_process_list(
        &mut self,
        request: crate::pb::ReportProcessListRequest,
    ) -> Result<crate::pb::ReportProcessListResponse>;

    ///
    /// Report execution output for a task.
    async fn report_task_output(
        &mut self,
        request: crate::pb::ReportTaskOutputRequest,
    ) -> Result<crate::pb::ReportTaskOutputResponse>;
}
