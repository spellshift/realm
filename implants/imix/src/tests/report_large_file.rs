use crate::agent::ImixAgent;
use anyhow::Result;
use eldritch::report::ReportLibrary;
use eldritch::report::std::StdReportLibrary;
use eldritch_agent::{Agent, Context};
use pb::c2::{ReportFileRequest, ReportFileResponse};
use pb::config::Config;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;
use tokio::sync::mpsc;
use transport::Transport;

use crate::shell::manager::ShellManagerMessage;
use crate::task::TaskRegistry;

#[derive(Clone)]
struct TestTransport {
    received_bytes: Arc<Mutex<usize>>,
    received_chunks: Arc<Mutex<usize>>,
}

impl Transport for TestTransport {
    fn init() -> Self {
        Self {
            received_bytes: Arc::new(Mutex::new(0)),
            received_chunks: Arc::new(Mutex::new(0)),
        }
    }

    fn new(_: Config) -> Result<Self> {
        Ok(Self::init())
    }

    async fn claim_tasks(
        &mut self,
        _: pb::c2::ClaimTasksRequest,
    ) -> Result<pb::c2::ClaimTasksResponse> {
        Ok(pb::c2::ClaimTasksResponse::default())
    }
    async fn fetch_asset(
        &mut self,
        _: pb::c2::FetchAssetRequest,
        _: Sender<pb::c2::FetchAssetResponse>,
    ) -> Result<()> {
        Ok(())
    }
    async fn report_credential(
        &mut self,
        _: pb::c2::ReportCredentialRequest,
    ) -> Result<pb::c2::ReportCredentialResponse> {
        Ok(pb::c2::ReportCredentialResponse::default())
    }

    async fn report_file(&mut self, rx: Receiver<ReportFileRequest>) -> Result<ReportFileResponse> {
        while let Ok(req) = rx.recv() {
            let mut bytes = self.received_bytes.lock().unwrap();
            let mut chunks = self.received_chunks.lock().unwrap();
            if let Some(chunk) = req.chunk {
                *bytes += chunk.chunk.len();
                *chunks += 1;
            }
        }
        Ok(ReportFileResponse {})
    }

    async fn report_process_list(
        &mut self,
        _: pb::c2::ReportProcessListRequest,
    ) -> Result<pb::c2::ReportProcessListResponse> {
        Ok(pb::c2::ReportProcessListResponse::default())
    }
    async fn report_output(
        &mut self,
        _: pb::c2::ReportOutputRequest,
    ) -> Result<pb::c2::ReportOutputResponse> {
        Ok(pb::c2::ReportOutputResponse::default())
    }
    async fn reverse_shell(
        &mut self,
        _: tokio::sync::mpsc::Receiver<pb::c2::ReverseShellRequest>,
        _: tokio::sync::mpsc::Sender<pb::c2::ReverseShellResponse>,
    ) -> Result<()> {
        Ok(())
    }
    async fn create_portal(
        &mut self,
        _: tokio::sync::mpsc::Receiver<pb::c2::CreatePortalRequest>,
        _: tokio::sync::mpsc::Sender<pb::c2::CreatePortalResponse>,
    ) -> Result<()> {
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
        vec!["test".into()]
    }
}

#[tokio::test]
async fn test_report_large_file_via_eldritch() {
    let file_size = 100 * 1024 * 1024; // 100MB
    let file = NamedTempFile::new().unwrap();
    file.as_file().set_len(file_size as u64).unwrap();
    let file_path = file.path().to_str().unwrap().to_string();

    let transport = TestTransport::init();
    let received_bytes = transport.received_bytes.clone();
    let received_chunks = transport.received_chunks.clone();

    let config = Config::default();
    let runtime_handle = tokio::runtime::Handle::current();
    let task_registry = Arc::new(TaskRegistry::new());
    let (shell_manager_tx, _shell_manager_rx) = mpsc::channel::<ShellManagerMessage>(1);

    let agent = Arc::new(ImixAgent::new(
        config,
        transport,
        runtime_handle,
        task_registry,
        shell_manager_tx,
    ));

    let context = Context::Task(pb::c2::TaskContext {
        task_id: 1,
        jwt: "test".to_string(),
    });

    let lib = StdReportLibrary::new(agent, context);

    // Run blocking operation on a blocking thread to avoid blocking async runtime
    let path_clone = file_path.clone();
    let result = tokio::task::spawn_blocking(move || lib.file(path_clone))
        .await
        .unwrap();

    assert!(result.is_ok());

    let total_bytes = *received_bytes.lock().unwrap();
    let total_chunks = *received_chunks.lock().unwrap();

    assert_eq!(total_bytes, file_size);
    // 100MB / 1MB chunk size = 100 chunks
    assert_eq!(total_chunks, 100);
}
