use crate::{PivotLibrary, std::StdPivotLibrary};
use alloc::collections::{BTreeMap, BTreeSet};
use eldritch_agent::Agent;
use pb::c2;
use std::sync::{Arc, Mutex};

// Mock Agent
struct MockAgent {
    // TODO: Determine if this can be simplified
    #[allow(clippy::type_complexity)]
    start_calls: Arc<Mutex<Vec<(i64, Option<String>)>>>,
    repl_calls: Arc<Mutex<Vec<i64>>>,
}

impl MockAgent {
    fn new() -> Self {
        Self {
            start_calls: Arc::new(Mutex::new(Vec::new())),
            repl_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Agent for MockAgent {
    fn fetch_asset(&self, _req: c2::FetchAssetRequest) -> Result<Vec<u8>, String> {
        Ok(vec![])
    }
    fn report_credential(
        &self,
        _req: c2::ReportCredentialRequest,
    ) -> Result<c2::ReportCredentialResponse, String> {
        Ok(c2::ReportCredentialResponse {})
    }
    fn report_file(&self, _req: c2::ReportFileRequest) -> Result<c2::ReportFileResponse, String> {
        Ok(c2::ReportFileResponse {})
    }
    fn report_process_list(
        &self,
        _req: c2::ReportProcessListRequest,
    ) -> Result<c2::ReportProcessListResponse, String> {
        Ok(c2::ReportProcessListResponse {})
    }
    fn report_task_output(
        &self,
        _req: c2::ReportTaskOutputRequest,
    ) -> Result<c2::ReportTaskOutputResponse, String> {
        Ok(c2::ReportTaskOutputResponse {})
    }
    fn reverse_shell(&self) -> Result<(), String> {
        Ok(())
    }
    fn start_reverse_shell(&self, task_id: i64, cmd: Option<String>) -> Result<(), String> {
        self.start_calls.lock().unwrap().push((task_id, cmd));
        Ok(())
    }
    fn claim_tasks(&self, _req: c2::ClaimTasksRequest) -> Result<c2::ClaimTasksResponse, String> {
        Ok(c2::ClaimTasksResponse { tasks: vec![] })
    }
    fn get_config(&self) -> Result<BTreeMap<String, String>, String> {
        Ok(BTreeMap::new())
    }

    fn get_transport(&self) -> Result<String, String> {
        Ok("mock".to_string())
    }
    fn set_transport(&self, _transport: String) -> Result<(), String> {
        Ok(())
    }
    fn list_transports(&self) -> Result<Vec<String>, String> {
        Ok(vec![])
    }
    fn get_callback_interval(&self) -> Result<u64, String> {
        Ok(5)
    }
    fn set_callback_interval(&self, _interval: u64) -> Result<(), String> {
        Ok(())
    }
    fn list_tasks(&self) -> Result<Vec<c2::Task>, String> {
        Ok(vec![])
    }
    fn stop_task(&self, _task_id: i64) -> Result<(), String> {
        Ok(())
    }
    fn start_repl_reverse_shell(&self, task_id: i64) -> Result<(), String> {
        self.repl_calls.lock().unwrap().push(task_id);
        Ok(())
    }
    fn set_callback_uri(&self, _uri: String) -> std::result::Result<(), String> {
        Ok(())
    }
    fn list_callback_uris(&self) -> std::result::Result<BTreeSet<String>, String> {
        Ok(BTreeSet::new())
    }
    fn get_active_callback_uri(&self) -> std::result::Result<String, String> {
        Ok(String::new())
    }
    fn get_next_callback_uri(&self) -> std::result::Result<String, String> {
        Ok(String::new())
    }
    fn add_callback_uri(&self, _uri: String) -> std::result::Result<(), String> {
        Ok(())
    }
    fn remove_callback_uri(&self, _uri: String) -> std::result::Result<(), String> {
        Ok(())
    }
    fn set_active_callback_uri(&self, _uri: String) -> std::result::Result<(), String> {
        Ok(())
    }
}

#[test]
fn test_reverse_shell_pty_delegation() {
    let agent = Arc::new(MockAgent::new());
    let task_id = 999;
    let lib = StdPivotLibrary::new(agent.clone(), task_id);

    // Test with command
    lib.reverse_shell_pty(Some("bash".to_string())).unwrap();

    let calls = agent.start_calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, task_id);
    assert_eq!(calls[0].1, Some("bash".to_string()));
}

#[test]
fn test_reverse_shell_pty_no_agent() {
    let lib = StdPivotLibrary::default();
    let result = lib.reverse_shell_pty(None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No agent available");
}

#[test]
fn test_reverse_shell_repl_delegation() {
    let agent = Arc::new(MockAgent::new());
    let task_id = 123;
    let lib = StdPivotLibrary::new(agent.clone(), task_id);

    lib.reverse_shell_repl().unwrap();

    let calls = agent.repl_calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], task_id);
}

#[test]
fn test_reverse_shell_repl_no_agent() {
    let lib = StdPivotLibrary::default();
    let result = lib.reverse_shell_repl();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No agent available");
}
