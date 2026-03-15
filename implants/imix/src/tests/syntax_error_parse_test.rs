use super::super::task::TaskRegistry;
use alloc::collections::{BTreeMap, BTreeSet};
use eldritch::agent::agent::Agent;
use eldritch_agent::Context;
use pb::c2;
use pb::c2::report_output_request;
use pb::eldritch::Tome;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

// Mock Agent specifically for TaskRegistry
struct MockAgent {
    output_reports: Arc<Mutex<Vec<c2::ReportOutputRequest>>>,
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
        _req: std::sync::mpsc::Receiver<c2::ReportFileRequest>,
    ) -> Result<c2::ReportFileResponse, String> {
        Ok(c2::ReportFileResponse {})
    }
    fn report_process_list(
        &self,
        _req: c2::ReportProcessListRequest,
    ) -> Result<c2::ReportProcessListResponse, String> {
        Ok(c2::ReportProcessListResponse {})
    }
    fn report_output(
        &self,
        req: c2::ReportOutputRequest,
    ) -> Result<c2::ReportOutputResponse, String> {
        self.output_reports.lock().unwrap().push(req);
        Ok(c2::ReportOutputResponse {})
    }
    fn create_portal(&self, _context: Context) -> Result<(), String> {
        Ok(())
    }
    fn start_reverse_shell(&self, _context: Context, _cmd: Option<String>) -> Result<(), String> {
        Ok(())
    }
    fn start_repl_reverse_shell(&self, _context: Context) -> Result<(), String> {
        Ok(())
    }
    fn claim_tasks(&self, _req: c2::ClaimTasksRequest) -> Result<c2::ClaimTasksResponse, String> {
        Ok(c2::ClaimTasksResponse {
            tasks: vec![],
            shell_tasks: vec![],
        })
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
async fn test_task_syntax_error_parse() {
    let agent = Arc::new(MockAgent::new());
    let task_id = 405;
    let code = "def my_func() {\n  return 1\n}"; // syntax error in eldritch (python-like)

    let task = c2::Task {
        id: task_id,
        tome: Some(Tome {
            eldritch: code.to_string(),
            ..Default::default()
        }),
        quest_name: "syntax_error_parse_test".to_string(),
        ..Default::default()
    };

    let registry = TaskRegistry::new();
    registry.spawn(task, agent.clone());

    tokio::time::sleep(Duration::from_secs(3)).await;

    let reports = agent.output_reports.lock().unwrap();

    // Check for error report
    let error_report = reports.iter().find(|r| {
        if let Some(report_output_request::Message::TaskOutput(m)) = &r.message {
            if let Some(o) = &m.output {
                return o.error.is_some();
            }
        }
        false
    });

    assert!(error_report.is_some(), "Should report error");
    if let Some(r) = error_report {
        if let Some(report_output_request::Message::TaskOutput(m)) = &r.message {
            if let Some(o) = &m.output {
                if let Some(e) = &o.error {
                    println!("Error message: {}", e.msg);
                    assert!(e.msg.contains("Error") || e.msg.contains("Lexer") || e.msg.contains("Syntax"));
                }
            }
        }
    }
}
