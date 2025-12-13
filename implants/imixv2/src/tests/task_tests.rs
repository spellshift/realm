use super::super::task::TaskRegistry;
use alloc::collections::{BTreeMap, BTreeSet};
use eldritch_libagent::agent::Agent;
use eldritchv2::pivot::ReplHandler;
use pb::c2;
use pb::eldritch::Tome;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use transport::SyncTransport;
use std::sync::mpsc::{Receiver, Sender};

struct MockAgent {
    output_reports: Arc<Mutex<Vec<c2::ReportTaskOutputRequest>>>,
}

impl MockAgent {
    fn new() -> Self {
        Self { output_reports: Arc::new(Mutex::new(Vec::new())) }
    }
}

impl Agent for MockAgent {
    fn get_config(&self) -> Result<BTreeMap<String, String>, String> { Ok(BTreeMap::new()) }
    fn get_transport(&self) -> Result<String, String> { Ok("mock".to_string()) }
    fn set_transport(&self, _t: String) -> Result<(), String> { Ok(()) }
    fn list_transports(&self) -> Result<Vec<String>, String> { Ok(vec!["mock".to_string()]) }
    fn get_callback_interval(&self) -> Result<u64, String> { Ok(5) }
    fn set_callback_interval(&self, _i: u64) -> Result<(), String> { Ok(()) }
    fn list_tasks(&self) -> Result<Vec<c2::Task>, String> { Ok(vec![]) }
    fn stop_task(&self, _t: i64) -> Result<(), String> { Ok(()) }
    fn set_callback_uri(&self, _u: String) -> Result<(), String> { Ok(()) }
    fn list_callback_uris(&self) -> Result<BTreeSet<String>, String> { Ok(BTreeSet::new()) }
    fn get_active_callback_uri(&self) -> Result<String, String> { Ok(String::new()) }
    fn get_next_callback_uri(&self) -> Result<String, String> { Ok(String::new()) }
    fn add_callback_uri(&self, _u: String) -> Result<(), String> { Ok(()) }
    fn remove_callback_uri(&self, _u: String) -> Result<(), String> { Ok(()) }
    fn set_active_callback_uri(&self, _u: String) -> Result<(), String> { Ok(()) }
}

impl SyncTransport for MockAgent {
    fn fetch_asset(&self, _r: c2::FetchAssetRequest) -> anyhow::Result<Vec<u8>> { Ok(vec![]) }
    fn report_credential(&self, _r: c2::ReportCredentialRequest) -> anyhow::Result<c2::ReportCredentialResponse> { Ok(c2::ReportCredentialResponse {}) }
    fn report_file(&self, _r: c2::ReportFileRequest) -> anyhow::Result<c2::ReportFileResponse> { Ok(c2::ReportFileResponse {}) }
    fn report_process_list(&self, _r: c2::ReportProcessListRequest) -> anyhow::Result<c2::ReportProcessListResponse> { Ok(c2::ReportProcessListResponse {}) }
    fn report_task_output(&self, req: c2::ReportTaskOutputRequest) -> anyhow::Result<c2::ReportTaskOutputResponse> {
        self.output_reports.lock().unwrap().push(req);
        Ok(c2::ReportTaskOutputResponse {})
    }
    fn reverse_shell(&self, _rx: Receiver<c2::ReverseShellRequest>, _tx: Sender<c2::ReverseShellResponse>) -> anyhow::Result<()> { Ok(()) }
    fn claim_tasks(&self, _r: c2::ClaimTasksRequest) -> anyhow::Result<c2::ClaimTasksResponse> { Ok(c2::ClaimTasksResponse { tasks: vec![] }) }
}

impl ReplHandler for MockAgent {
    fn start_repl_reverse_shell(&self, _t: i64) -> Result<(), String> { Ok(()) }
}

#[tokio::test]
async fn test_task_registry_spawn() {
    let agent = Arc::new(MockAgent::new());
    let task_id = 123;
    let task = c2::Task {
        id: task_id,
        tome: Some(Tome { eldritch: "print(\"Hello World\")".to_string(), ..Default::default() }),
        quest_name: "test_quest".to_string(),
    };
    let registry = TaskRegistry::new();
    registry.spawn(task, agent.clone(), agent.clone(), Some(agent.clone()));

    tokio::time::sleep(Duration::from_secs(2)).await;
    let reports = agent.output_reports.lock().unwrap();
    assert!(!reports.is_empty(), "Should have reported output");
    let has_output = reports.iter().any(|r| r.output.as_ref().map(|o| o.output.contains("Hello World")).unwrap_or(false));
    assert!(has_output);
}

#[tokio::test]
async fn test_task_streaming_output() {
    let agent = Arc::new(MockAgent::new());
    let task_id = 456;
    let code = "print(\"Chunk 1\")\nprint(\"Chunk 2\")";
    let task = c2::Task {
        id: task_id,
        tome: Some(Tome { eldritch: code.to_string(), ..Default::default() }),
        quest_name: "streaming_test".to_string(),
    };
    let registry = TaskRegistry::new();
    registry.spawn(task, agent.clone(), agent.clone(), Some(agent.clone()));

    tokio::time::sleep(Duration::from_secs(3)).await;
    let reports = agent.output_reports.lock().unwrap();
    let outputs: Vec<String> = reports.iter().filter_map(|r| r.output.as_ref().map(|o| o.output.clone())).filter(|s| !s.is_empty()).collect();
    assert!(!outputs.is_empty());
    let combined = outputs.join("");
    assert!(combined.contains("Chunk 1"));
    assert!(combined.contains("Chunk 2"));
}

#[tokio::test]
async fn test_task_streaming_error() {
    let agent = Arc::new(MockAgent::new());
    let task_id = 789;
    let code = "print(\"Before Error\")\nx = 1 / 0";
    let task = c2::Task {
        id: task_id,
        tome: Some(Tome { eldritch: code.to_string(), ..Default::default() }),
        quest_name: "error_test".to_string(),
    };
    let registry = TaskRegistry::new();
    registry.spawn(task, agent.clone(), agent.clone(), Some(agent.clone()));

    tokio::time::sleep(Duration::from_secs(3)).await;
    let reports = agent.output_reports.lock().unwrap();
    let outputs: Vec<String> = reports.iter().filter_map(|r| r.output.as_ref().map(|o| o.output.clone())).filter(|s| !s.is_empty()).collect();
    assert!(outputs.iter().any(|s| s.contains("Before Error")));
    let error_report = reports.iter().find(|r| r.output.as_ref().map(|o| o.error.is_some()).unwrap_or(false));
    assert!(error_report.is_some());
}

#[tokio::test]
async fn test_task_registry_list_and_stop() {
    let agent = Arc::new(MockAgent::new());
    let task_id = 999;
    let task = c2::Task {
        id: task_id,
        tome: Some(Tome { eldritch: "print(\"x=1\")".to_string(), ..Default::default() }),
        quest_name: "list_stop_quest".to_string(),
    };
    let registry = TaskRegistry::new();
    registry.spawn(task, agent.clone(), agent.clone(), Some(agent.clone()));
    let _list = registry.list();
    registry.stop(task_id);
    let tasks_after = registry.list();
    assert!(!tasks_after.iter().any(|t| t.id == task_id));
}
