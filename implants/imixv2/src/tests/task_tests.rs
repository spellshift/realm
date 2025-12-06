use super::super::task::TaskRegistry;
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
    fn report_file(
        &self,
        _req: c2::ReportFileRequest,
    ) -> Result<c2::ReportFileResponse, String> {
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
    fn reverse_shell(&self) -> Result<(), String> {
        Ok(())
    }
    fn start_reverse_shell(&self, _task_id: i64, _cmd: Option<String>) -> Result<(), String> {
        Ok(())
    }
    fn start_repl_reverse_shell(&self, _task_id: i64) -> Result<(), String> {
        Ok(())
    }
    fn claim_tasks(
        &self,
        _req: c2::ClaimTasksRequest,
    ) -> Result<c2::ClaimTasksResponse, String> {
        Ok(c2::ClaimTasksResponse { tasks: vec![] })
    }
    fn get_transport(&self) -> Result<String, String> {
        Ok("mock".to_string())
    }
    fn set_transport(&self, _transport: String) -> Result<(), String> {
        Ok(())
    }
    fn add_transport(&self, _transport: String, _config: String) -> Result<(), String> {
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
}

#[test]
fn test_task_registry_spawn() {
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

    // Give it a moment to start
    std::thread::sleep(Duration::from_millis(100));

    // Wait a bit more for execution
    std::thread::sleep(Duration::from_secs(1));

    let reports = agent.output_reports.lock().unwrap();
    assert!(!reports.is_empty(), "Should have reported output");

    let found = reports.iter().any(|r| {
        if let Some(output) = &r.output {
            output.id == task_id && output.output.contains("Hello World")
        } else {
            false
        }
    });

    assert!(found, "Should have reported output containing 'Hello World'");
}

#[test]
fn test_task_registry_list_and_stop() {
    let agent = Arc::new(MockAgent::new());
    let task_id = 456;
    let task = c2::Task {
        id: task_id,
        tome: Some(Tome {
            eldritch: "import time; time.sleep(2)".to_string(),
            ..Default::default()
        }),
        quest_name: "long_quest".to_string(),
    };

    let registry = TaskRegistry::new();
    registry.spawn(task, agent.clone());

    // Check repeatedly if the task is running to avoid race conditions
    let mut running = false;
    for _ in 0..10 {
        if registry.list().iter().any(|t| t.id == task_id) {
            running = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(running, "Task should be in list");

    registry.stop(task_id);
    let tasks_after = registry.list();
    assert!(!tasks_after.iter().any(|t| t.id == task_id), "Task should be removed from list");
}
