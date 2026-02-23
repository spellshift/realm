use crate::agent::ImixAgent;
use crate::task::TaskRegistry;
use pb::c2::{ReportFileRequest, ReportFileResponse};
use pb::config::Config;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use tokio::sync::RwLock;
use transport::Transport;

// Simple TestTransport to avoid mockall issues with cloning and expectations
#[derive(Clone)]
struct TestTransport {
    pub received_chunks: Arc<RwLock<Vec<usize>>>,
}

impl TestTransport {
    fn new() -> Self {
        Self {
            received_chunks: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Transport for TestTransport {
    fn init() -> Self {
        Self::new()
    }

    fn new(_config: Config) -> anyhow::Result<Self> {
        Ok(Self::new())
    }

    async fn claim_tasks(
        &mut self,
        _request: pb::c2::ClaimTasksRequest,
    ) -> anyhow::Result<pb::c2::ClaimTasksResponse> {
        Ok(pb::c2::ClaimTasksResponse::default())
    }

    async fn fetch_asset(
        &mut self,
        _request: pb::c2::FetchAssetRequest,
        _sender: std::sync::mpsc::Sender<pb::c2::FetchAssetResponse>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn report_credential(
        &mut self,
        _request: pb::c2::ReportCredentialRequest,
    ) -> anyhow::Result<pb::c2::ReportCredentialResponse> {
        Ok(pb::c2::ReportCredentialResponse::default())
    }

    async fn report_file(
        &mut self,
        request: Receiver<ReportFileRequest>,
    ) -> anyhow::Result<ReportFileResponse> {
        while let Ok(req) = request.recv() {
            if let Some(chunk) = req.chunk {
                let mut chunks = self.received_chunks.write().await;
                chunks.push(chunk.chunk.len());
            }
        }
        Ok(ReportFileResponse {})
    }

    async fn report_process_list(
        &mut self,
        _request: pb::c2::ReportProcessListRequest,
    ) -> anyhow::Result<pb::c2::ReportProcessListResponse> {
        Ok(pb::c2::ReportProcessListResponse::default())
    }

    async fn report_output(
        &mut self,
        _request: pb::c2::ReportOutputRequest,
    ) -> anyhow::Result<pb::c2::ReportOutputResponse> {
        Ok(pb::c2::ReportOutputResponse::default())
    }

    async fn reverse_shell(
        &mut self,
        _rx: tokio::sync::mpsc::Receiver<pb::c2::ReverseShellRequest>,
        _tx: tokio::sync::mpsc::Sender<pb::c2::ReverseShellResponse>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn create_portal(
        &mut self,
        _rx: tokio::sync::mpsc::Receiver<pb::c2::CreatePortalRequest>,
        _tx: tokio::sync::mpsc::Sender<pb::c2::CreatePortalResponse>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_type(&mut self) -> pb::c2::transport::Type {
        pb::c2::transport::Type::TransportUnspecified
    }

    fn is_active(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "test"
    }

    fn list_available(&self) -> Vec<String> {
        vec!["test".to_string()]
    }
}

#[tokio::test]
async fn test_report_large_file_via_eldritch() {
    // 1. Create a temporary file larger than 1MB (e.g. 5MB)
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("large_file.bin");
    let file_size = 5 * 1024 * 1024; // 5MB
    let content = vec![0x41; file_size]; // 'A'
    std::fs::write(&file_path, &content).unwrap();

    // 2. Setup Transport
    let transport = TestTransport::new();
    let received_chunks_ref = transport.received_chunks.clone();

    // 3. Setup ImixAgent
    let runtime_handle = tokio::runtime::Handle::current();
    let task_registry = Arc::new(TaskRegistry::new());
    let (shell_manager_tx, _) = tokio::sync::mpsc::channel(100);

    let config = Config::default();

    let agent = Arc::new(ImixAgent::new(
        config,
        transport,
        runtime_handle,
        task_registry,
        shell_manager_tx,
    ));

    // 4. Call report.file via eldritch library
    let context = eldritch_agent::Context::Task(pb::c2::TaskContext {
        task_id: 1,
        jwt: "test".into(),
    });

    let path_str = file_path.to_str().unwrap().to_string();

    // Run in spawn_blocking because eldritch functions block
    let agent_clone = agent.clone();
    let result = tokio::task::spawn_blocking(move || {
        eldritch::report::std::file_impl::file(agent_clone, context, path_str)
    })
    .await
    .unwrap();

    assert!(result.is_ok(), "Report file failed: {:?}", result.err());

    // 5. Verify results
    let chunks = received_chunks_ref.read().await;
    let total_bytes: usize = chunks.iter().sum();

    // Clean up
    let _ = std::fs::remove_file(file_path);

    assert_eq!(
        total_bytes, file_size,
        "Total bytes received should match file size"
    );
    assert_eq!(chunks.len(), 5, "Should have received 5 chunks of 1MB each");
    for &chunk_size in chunks.iter() {
        assert_eq!(chunk_size, 1024 * 1024, "Each chunk should be 1MB");
    }
}
