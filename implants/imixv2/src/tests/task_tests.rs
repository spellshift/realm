use super::super::task::TaskRegistry;
use alloc::collections::{BTreeMap, BTreeSet};
use eldritch_libagent::agent::Agent;
use pb::c2;
use pb::eldritch::Tome;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

// Mock Agent specifically for TaskRegistry
struct MockAgent {
    output_reports: Arc<Mutex<Vec<c2::ReportTaskOutputRequest>>>,
}

impl MockAgent {
    fn new() -> Self {
        Self {
            output_reports: Arc::new(Mutex::new(Vec::new())),
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
        req: c2::ReportTaskOutputRequest,
    ) -> Result<c2::ReportTaskOutputResponse, String> {
        self.output_reports.lock().unwrap().push(req);
        Ok(c2::ReportTaskOutputResponse {})
    }
    fn start_reverse_shell(&self, _task_id: i64, _cmd: Option<String>) -> Result<(), String> {
        Ok(())
    }
    fn start_repl_reverse_shell(&self, _task_id: i64) -> Result<(), String> {
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
        Ok(vec!["mock".to_string()])
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
}

#[tokio::test]
async fn test_task_registry_spawn() {
    let agent = Arc::new(MockAgent::new());
    let task_id = 123;
    let task = c2::Task {
        id: task_id,
        tome: Some(Tome {
            eldritch: "print(\"Hello World\")".to_string(),
            ..Default::default()
        }),
        quest_name: "test_quest".to_string(),
    };

    let registry = TaskRegistry::new();
    registry.spawn(task, agent.clone());

    // Wait a bit more for execution
    tokio::time::sleep(Duration::from_secs(2)).await;

    let reports = agent.output_reports.lock().unwrap();
    assert!(!reports.is_empty(), "Should have reported output");

    // Check for Hello World
    let has_output = reports.iter().any(|r| {
        r.output
            .as_ref()
            .map(|o| o.output.contains("Hello World"))
            .unwrap_or(false)
    });
    assert!(
        has_output,
        "Should have found report containing 'Hello World'"
    );

    // Check completion
    let has_finished = reports.iter().any(|r| {
        r.output
            .as_ref()
            .map(|o| o.exec_finished_at.is_some())
            .unwrap_or(false)
    });
    assert!(has_finished, "Should have marked task as finished");
}

#[tokio::test]
async fn test_task_streaming_output() {
    let agent = Arc::new(MockAgent::new());
    let task_id = 456;
    // Removed indentation and loops to avoid parser errors in string literal
    let code = "print(\"Chunk 1\")\nprint(\"Chunk 2\")";
    println!("Code: {:?}", code);

    let task = c2::Task {
        id: task_id,
        tome: Some(Tome {
            eldritch: code.to_string(),
            ..Default::default()
        }),
        quest_name: "streaming_test".to_string(),
    };

    let registry = TaskRegistry::new();
    registry.spawn(task, agent.clone());

    tokio::time::sleep(Duration::from_secs(3)).await;

    let reports = agent.output_reports.lock().unwrap();

    // Debug output
    println!("Reports count: {}", reports.len());
    for r in reports.iter() {
        println!("Report: {:?}", r);
    }

    let outputs: Vec<String> = reports
        .iter()
        .filter_map(|r| r.output.as_ref().map(|o| o.output.clone()))
        .filter(|s| !s.is_empty())
        .collect();

    assert!(!outputs.is_empty(), "Should have at least one output.");

    let combined = outputs.join("");
    assert!(combined.contains("Chunk 1"), "Missing Chunk 1");
    assert!(combined.contains("Chunk 2"), "Missing Chunk 2");
}

#[tokio::test]
async fn test_task_streaming_error() {
    let agent = Arc::new(MockAgent::new());
    let task_id = 789;
    let code = "print(\"Before Error\")\nx = 1 / 0";
    println!("Code: {:?}", code);

    let task = c2::Task {
        id: task_id,
        tome: Some(Tome {
            eldritch: code.to_string(),
            ..Default::default()
        }),
        quest_name: "error_test".to_string(),
    };

    let registry = TaskRegistry::new();
    registry.spawn(task, agent.clone());

    tokio::time::sleep(Duration::from_secs(3)).await;

    let reports = agent.output_reports.lock().unwrap();

    // Debug
    println!("Reports count: {}", reports.len());
    for r in reports.iter() {
        println!("Report: {:?}", r);
    }

    let outputs: Vec<String> = reports
        .iter()
        .filter_map(|r| r.output.as_ref().map(|o| o.output.clone()))
        .filter(|s| !s.is_empty())
        .collect();

    assert!(
        outputs.iter().any(|s| s.contains("Before Error")),
        "Should contain pre-error output"
    );

    // Check for error report
    let error_report = reports.iter().find(|r| {
        r.output
            .as_ref()
            .map(|o| o.error.is_some())
            .unwrap_or(false)
    });
    assert!(error_report.is_some(), "Should report error");
}

#[tokio::test]
async fn test_task_registry_list_and_stop() {
    let agent = Arc::new(MockAgent::new());
    let task_id = 999;
    let task = c2::Task {
        id: task_id,
        tome: Some(Tome {
            eldritch: "print(\"x=1\")".to_string(),
            ..Default::default()
        }),
        quest_name: "list_stop_quest".to_string(),
    };

    let registry = TaskRegistry::new();
    registry.spawn(task, agent.clone());

    // Check list immediately
    let _list = registry.list();

    registry.stop(task_id);
    let tasks_after = registry.list();
    assert!(
        !tasks_after.iter().any(|t| t.id == task_id),
        "Task should be removed from list"
    );
}
