use crate::AgentLibrary;
use crate::agent::Agent;
use crate::std::StdAgentLibrary;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use eldritch_core::Value;
use std::sync::RwLock;
use std::thread;

#[derive(Clone)]
struct MockAgent {
    config: Arc<RwLock<BTreeMap<String, String>>>,
}

impl MockAgent {
    fn new() -> Self {
        let mut map = BTreeMap::new();
        map.insert("key".to_string(), "value".to_string());
        map.insert("interval".to_string(), "5".to_string());
        Self {
            config: Arc::new(RwLock::new(map)),
        }
    }
}

impl Agent for MockAgent {
    fn get_config(&self) -> Result<BTreeMap<String, String>, String> {
        Ok(self.config.read().unwrap().clone())
    }

    fn set_callback_interval(&self, interval: u64) -> Result<(), String> {
        let mut cfg = self.config.write().unwrap();
        cfg.insert("interval".to_string(), interval.to_string());
        Ok(())
    }

    // Unused stubs
    fn fetch_asset(&self, _: pb::c2::FetchAssetRequest) -> Result<Vec<u8>, String> {
        Err("".into())
    }
    fn report_credential(
        &self,
        _: pb::c2::ReportCredentialRequest,
    ) -> Result<pb::c2::ReportCredentialResponse, String> {
        Err("".into())
    }
    fn report_file(
        &self,
        _: pb::c2::ReportFileRequest,
    ) -> Result<pb::c2::ReportFileResponse, String> {
        Err("".into())
    }
    fn report_process_list(
        &self,
        _: pb::c2::ReportProcessListRequest,
    ) -> Result<pb::c2::ReportProcessListResponse, String> {
        Err("".into())
    }
    fn report_task_output(
        &self,
        _: pb::c2::ReportTaskOutputRequest,
    ) -> Result<pb::c2::ReportTaskOutputResponse, String> {
        Err("".into())
    }
    fn create_portal(&self, _task_id: i64) -> Result<(), String> {
        Err("".into())
    }
    fn start_reverse_shell(&self, _: i64, _: Option<String>) -> Result<(), String> {
        Err("".into())
    }
    fn start_repl_reverse_shell(&self, _: i64) -> Result<(), String> {
        Err("".into())
    }
    fn claim_tasks(
        &self,
        _: pb::c2::ClaimTasksRequest,
    ) -> Result<pb::c2::ClaimTasksResponse, String> {
        Err("".into())
    }
    fn get_transport(&self) -> Result<String, String> {
        Err("".into())
    }
    fn set_transport(&self, _: String) -> Result<(), String> {
        Err("".into())
    }
    fn list_transports(&self) -> Result<Vec<String>, String> {
        Err("".into())
    }
    fn get_callback_interval(&self) -> Result<u64, String> {
        Err("".into())
    }
    fn set_callback_uri(&self, _: String) -> Result<(), String> {
        Err("".into())
    }
    fn list_callback_uris(&self) -> Result<BTreeSet<String>, String> {
        Err("".into())
    }
    fn get_active_callback_uri(&self) -> Result<String, String> {
        Err("".into())
    }
    fn get_next_callback_uri(&self) -> Result<String, String> {
        Err("".into())
    }
    fn add_callback_uri(&self, _: String) -> Result<(), String> {
        Err("".into())
    }
    fn remove_callback_uri(&self, _: String) -> Result<(), String> {
        Err("".into())
    }
    fn list_tasks(&self) -> Result<Vec<pb::c2::Task>, String> {
        Err("".into())
    }
    fn stop_task(&self, _: i64) -> Result<(), String> {
        Err("".into())
    }
}

#[test]
fn test_get_config() {
    let agent = Arc::new(MockAgent::new());
    let lib = StdAgentLibrary::new(agent, 1);

    let config = lib.get_config().unwrap();
    assert_eq!(config.get("key"), Some(&Value::String("value".to_string())));
    assert_eq!(config.get("interval"), Some(&Value::Int(5)));
}

#[test]
fn test_concurrent_access() {
    let agent = Arc::new(MockAgent::new());
    let lib = StdAgentLibrary::new(agent.clone(), 1);
    let lib = Arc::new(lib);

    let mut handles = vec![];

    // Reader threads
    for _ in 0..10 {
        let lib_clone = lib.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                let config = lib_clone.get_config().unwrap();
                assert!(config.contains_key("key"));
                assert!(config.contains_key("interval"));
            }
        }));
    }

    // Writer thread
    let agent_clone = agent.clone();
    handles.push(thread::spawn(move || {
        for i in 0..100 {
            let _ = agent_clone.set_callback_interval(i as u64);
        }
    }));

    for h in handles {
        h.join().unwrap();
    }

    // Verify final state
    let config = lib.get_config().unwrap();
    // The last written value depends on thread scheduling, but it should be valid int
    if let Some(Value::Int(val)) = config.get("interval") {
        assert!(*val >= 0 && *val <= 100);
    } else {
        panic!("Interval should be an int");
    }
}
