use crate::agent::ImixAgent;
use crate::task::TaskRegistry;
use eldritch::report::std::file_impl;
use eldritch_agent::{Agent, Context};
use pb::c2::{
    AvailableTransports, Beacon, ClaimTasksRequest, ClaimTasksResponse, CreatePortalRequest,
    CreatePortalResponse, FetchAssetRequest, FetchAssetResponse, ReportCredentialRequest,
    ReportCredentialResponse, ReportFileRequest, ReportFileResponse, ReportOutputRequest,
    ReportOutputResponse, ReportProcessListRequest, ReportProcessListResponse, ReverseShellRequest,
    ReverseShellResponse, TaskContext, Transport as C2Transport,
};
use pb::config::Config;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use transport::Transport;

#[derive(Clone)]
struct FakeTransport {
    received_chunks: Arc<Mutex<usize>>,
}

impl FakeTransport {
    fn new() -> Self {
        Self {
            received_chunks: Arc::new(Mutex::new(0)),
        }
    }
}

impl Transport for FakeTransport {
    fn init() -> Self {
        FakeTransport::new()
    }

    fn new(_config: Config) -> anyhow::Result<Self> {
        Ok(FakeTransport::new())
    }

    async fn claim_tasks(
        &mut self,
        _request: ClaimTasksRequest,
    ) -> anyhow::Result<ClaimTasksResponse> {
        Ok(ClaimTasksResponse::default())
    }

    async fn fetch_asset(
        &mut self,
        _request: FetchAssetRequest,
        _sender: Sender<FetchAssetResponse>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn report_credential(
        &mut self,
        _request: ReportCredentialRequest,
    ) -> anyhow::Result<ReportCredentialResponse> {
        Ok(ReportCredentialResponse::default())
    }

    async fn report_file(
        &mut self,
        request: Receiver<ReportFileRequest>,
    ) -> anyhow::Result<ReportFileResponse> {
        let mut count = 0;
        while let Ok(_) = request.recv() {
            count += 1;
        }
        *self.received_chunks.lock().unwrap() += count;
        Ok(ReportFileResponse::default())
    }

    async fn report_process_list(
        &mut self,
        _request: ReportProcessListRequest,
    ) -> anyhow::Result<ReportProcessListResponse> {
        Ok(ReportProcessListResponse::default())
    }

    async fn report_output(
        &mut self,
        _request: ReportOutputRequest,
    ) -> anyhow::Result<ReportOutputResponse> {
        Ok(ReportOutputResponse::default())
    }

    async fn reverse_shell(
        &mut self,
        _rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        _tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn create_portal(
        &mut self,
        _rx: tokio::sync::mpsc::Receiver<CreatePortalRequest>,
        _tx: tokio::sync::mpsc::Sender<CreatePortalResponse>,
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
        "fake"
    }

    fn list_available(&self) -> Vec<String> {
        vec!["fake".to_string()]
    }
}

#[tokio::test]
async fn test_report_large_file_via_eldritch() {
    // 1. Setup Fake Transport
    let fake_transport = FakeTransport::new();
    let received_chunks = fake_transport.received_chunks.clone();

    // 2. Create Dummy Large File
    let file_path = "large_test_file.dat";
    let file_size = 100 * 1024 * 1024; // 100MB
    {
        let file = std::fs::File::create(file_path).unwrap();
        file.set_len(file_size as u64).unwrap();
    }

    // 3. Setup ImixAgent
    let config = Config {
        info: Some(Beacon {
            available_transports: Some(AvailableTransports {
                transports: vec![C2Transport {
                    uri: "http://localhost".to_string(),
                    ..Default::default()
                }],
                active_index: 0,
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let task_registry = Arc::new(TaskRegistry::new());
    let (shell_tx, _) = tokio::sync::mpsc::channel(100);

    let agent = ImixAgent::new(
        config,
        fake_transport,
        tokio::runtime::Handle::current(),
        task_registry,
        shell_tx,
    );
    let agent = Arc::new(agent);

    // 4. Call file report
    let context = Context::Task(TaskContext {
        task_id: 1,
        jwt: "jwt".to_string(),
    });

    let agent_clone = agent.clone();
    let file_path_str = file_path.to_string();

    let result = std::thread::spawn(move || {
        file_impl::file(agent_clone, context, file_path_str)
    })
    .join()
    .unwrap();

    // Cleanup first to ensure file removal even if assertion fails (best effort)
    std::fs::remove_file(file_path).unwrap();

    assert!(result.is_ok(), "Report file failed: {:?}", result.err());

    // 5. Verify chunks
    // 100MB / 1MB = 100 chunks.
    let count = *received_chunks.lock().unwrap();
    assert_eq!(count, 100, "Should have received 100 chunks");
}
