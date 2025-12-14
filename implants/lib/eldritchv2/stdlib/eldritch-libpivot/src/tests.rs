use crate::{PivotLibrary, ReplHandler, std::StdPivotLibrary};
use alloc::collections::{BTreeMap, BTreeSet};
use pb::c2;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use transport::SyncTransport;

struct MockTransport {
    calls: Arc<Mutex<Vec<String>>>,
}
impl MockTransport {
    fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }
}
impl SyncTransport for MockTransport {
    fn fetch_asset(&self, _r: c2::FetchAssetRequest) -> anyhow::Result<Vec<u8>> {
        Ok(vec![])
    }
    fn report_credential(
        &self,
        _r: c2::ReportCredentialRequest,
    ) -> anyhow::Result<c2::ReportCredentialResponse> {
        Ok(c2::ReportCredentialResponse {})
    }
    fn report_file(&self, _r: c2::ReportFileRequest) -> anyhow::Result<c2::ReportFileResponse> {
        Ok(c2::ReportFileResponse {})
    }
    fn report_process_list(
        &self,
        _r: c2::ReportProcessListRequest,
    ) -> anyhow::Result<c2::ReportProcessListResponse> {
        Ok(c2::ReportProcessListResponse {})
    }
    fn report_task_output(
        &self,
        _r: c2::ReportTaskOutputRequest,
    ) -> anyhow::Result<c2::ReportTaskOutputResponse> {
        Ok(c2::ReportTaskOutputResponse {})
    }
    fn reverse_shell(
        &self,
        _rx: Receiver<c2::ReverseShellRequest>,
        _tx: Sender<c2::ReverseShellResponse>,
    ) -> anyhow::Result<()> {
        self.calls.lock().unwrap().push("reverse_shell".to_string());
        Ok(())
    }
    fn claim_tasks(&self, _r: c2::ClaimTasksRequest) -> anyhow::Result<c2::ClaimTasksResponse> {
        Ok(c2::ClaimTasksResponse { tasks: vec![] })
    }
}

struct MockReplHandler {
    calls: Arc<Mutex<Vec<i64>>>,
}
impl ReplHandler for MockReplHandler {
    fn start_repl_reverse_shell(&self, task_id: i64) -> Result<(), String> {
        self.calls.lock().unwrap().push(task_id);
        Ok(())
    }
}

#[test]
#[ignore]
fn test_reverse_shell_pty_delegation() {
    let transport = Arc::new(MockTransport::new());
    let task_id = 999;
    let lib = StdPivotLibrary::new(transport.clone(), None, task_id);

    let res = lib.reverse_shell_pty(Some("echo test".to_string()));
    assert!(res.is_ok());

    let calls = transport.calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], "reverse_shell");
}

#[test]
fn test_reverse_shell_repl_delegation() {
    let transport = Arc::new(MockTransport::new());
    let repl_handler = Arc::new(MockReplHandler {
        calls: Arc::new(Mutex::new(Vec::new())),
    });
    let task_id = 123;
    let lib = StdPivotLibrary::new(transport, Some(repl_handler.clone()), task_id);

    lib.reverse_shell_repl().unwrap();

    let calls = repl_handler.calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], task_id);
}
