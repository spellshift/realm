use crate::{PivotLibrary, ReplHandler, std::StdPivotLibrary};
use pb::c2;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use transport::SyncTransport;
use eldritch_agent::{Agent, SubtaskFuture};
use alloc::collections::{BTreeMap, BTreeSet};

struct MockAgent {
    calls: Arc<Mutex<Vec<String>>>,
    should_block: bool,
    transport: Arc<MockTransport>,
}

impl MockAgent {
    fn new(transport: Arc<MockTransport>) -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
            should_block: false,
            transport,
        }
    }
}

impl Agent for MockAgent {
    fn get_config(&self) -> Result<BTreeMap<String, String>, String> { Err("Not impl".into()) }
    fn get_transport(&self) -> Result<String, String> { Ok("mock".into()) }
    fn set_transport(&self, _transport: String) -> Result<(), String> { Ok(()) }
    fn list_transports(&self) -> Result<Vec<String>, String> { Ok(vec![]) }
    fn get_callback_interval(&self) -> Result<u64, String> { Ok(0) }
    fn set_callback_interval(&self, _interval: u64) -> Result<(), String> { Ok(()) }
    fn list_tasks(&self) -> Result<Vec<c2::Task>, String> { Ok(vec![]) }
    fn stop_task(&self, _task_id: i64) -> Result<(), String> { Ok(()) }
    fn set_callback_uri(&self, _uri: String) -> Result<(), String> { Ok(()) }
    fn list_callback_uris(&self) -> Result<BTreeSet<String>, String> { Ok(BTreeSet::new()) }
    fn get_active_callback_uri(&self) -> Result<String, String> { Ok("".into()) }
    fn get_next_callback_uri(&self) -> Result<String, String> { Ok("".into()) }
    fn add_callback_uri(&self, _uri: String) -> Result<(), String> { Ok(()) }
    fn remove_callback_uri(&self, _uri: String) -> Result<(), String> { Ok(()) }
    fn set_active_callback_uri(&self, _uri: String) -> Result<(), String> { Ok(()) }
    fn spawn_subtask(&self, _task_id: i64, _name: String, future: SubtaskFuture) -> Result<(), String> {
        // Just execute the future immediately for testing
        // Note: this assumes the future doesn't depend on runtime features not available here
        // But reverse_shell_pty_impl spawns threads, which is fine in test
        let waker = futures::task::noop_waker();
        let mut cx = core::task::Context::from_waker(&waker);
        let mut f = future;
        let _ = f.as_mut().poll(&mut cx);
        Ok(())
    }
    fn get_sync_transport(&self) -> Option<Arc<dyn SyncTransport>> {
        Some(self.transport.clone())
    }
}

struct MockTransport {
    calls: Arc<Mutex<Vec<String>>>,
    // Add a latch to simulate blocking if needed, but for now simple logging is enough
    // to check delegation. To verify it blocks, we'd need more complex threading logic
    // in the test. But primarily we want to ensure it calls the method.
    // The bug was in SyncTransportAdapter, not SyncTransport trait itself.
    // However, we want to ensure reverse_shell_pty works with a blocking transport.
    should_block: bool,
}
impl MockTransport {
    fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
            should_block: false,
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
        if self.should_block {
             // Simulate a short session
             std::thread::sleep(std::time::Duration::from_millis(100));
        }
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
fn test_reverse_shell_pty_delegation() {
    let mut transport_mock = MockTransport::new();
    transport_mock.should_block = true;
    let transport = Arc::new(transport_mock);
    let agent = Arc::new(MockAgent::new(transport.clone()));

    let task_id = 999;
    let lib = StdPivotLibrary::new(agent, None, task_id);

    // Use "sh" (or similar) which should exist. We don't need it to do anything specific,
    // just start and eventually be killed or exit when we close channels.
    // Since we mock the transport to close after 100ms (by returning from reverse_shell),
    // the input loop in reverse_shell_pty_impl will eventually see a channel close (when in_tx is dropped)
    // or we rely on child.kill() at the end.
    // Actually, if we pass a shell, it waits for input.
    // If we use "true", it exits immediately.
    #[cfg(not(target_os = "windows"))]
    let cmd = "true";
    #[cfg(target_os = "windows")]
    let cmd = "cmd.exe /c exit 0";

    let res = lib.reverse_shell_pty(Some(cmd.to_string()));
    if let Err(e) = &res {
        println!("Error: {:?}", e);
    }
    assert!(res.is_ok());

    let calls = transport.calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], "reverse_shell");
}

#[test]
fn test_reverse_shell_repl_delegation() {
    let transport = Arc::new(MockTransport::new());
    let agent = Arc::new(MockAgent::new(transport));
    let repl_handler = Arc::new(MockReplHandler {
        calls: Arc::new(Mutex::new(Vec::new())),
    });
    let task_id = 123;
    let lib = StdPivotLibrary::new(agent, Some(repl_handler.clone()), task_id);

    lib.reverse_shell_repl().unwrap();

    let calls = repl_handler.calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], task_id);
}
