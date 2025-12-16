extern crate alloc;

use eldritchv2::{agent::AgentLibrary};
use eldritch_libagent::agent::Agent;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use std::sync::RwLock;
use std::thread;

#[derive(Clone)]
pub struct GolemAgent {
    config: Arc<RwLock<BTreeMap<String, String>>>,
}

impl GolemAgent {
    pub fn new() -> Self {
        let mut map = BTreeMap::new();
        map.insert("interval".to_string(), "5".to_string());
        Self {
            config: Arc::new(RwLock::new(map)),
        }
    }
}

impl Agent for GolemAgent {
    fn get_config(&self) -> Result<BTreeMap<String, String>, String> {
        Ok(self.config.read().unwrap().clone())
    }

    // This is the one golem cares about
    fn fetch_asset(&self, req: pb::c2::FetchAssetRequest) -> Result<Vec<u8>, String> {
        eprintln!("got an asset request: {}\n", req.name);
        Err("".into())
    }

    // TODO figure these out
    fn report_task_output(
        &self,
        _: pb::c2::ReportTaskOutputRequest,
    ) -> Result<pb::c2::ReportTaskOutputResponse, String> {
        Err("".into())
    }

    fn reverse_shell(&self) -> Result<(), String> {
        Err("".into())
    }
    fn start_reverse_shell(&self, _: i64, _: Option<String>) -> Result<(), String> {
        Err("".into())
    }
    fn start_repl_reverse_shell(&self, _: i64) -> Result<(), String> {
        Err("".into())
    }

    // Unused stubs

    fn set_callback_interval(&self, interval: u64) -> Result<(), String> {
        let mut cfg = self.config.write().unwrap();
        cfg.insert("interval".to_string(), interval.to_string());
        Ok(())
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
    fn set_active_callback_uri(&self, _: String) -> Result<(), String> {
        Err("".into())
    }
    fn list_tasks(&self) -> Result<Vec<pb::c2::Task>, String> {
        Err("".into())
    }
    fn stop_task(&self, _: i64) -> Result<(), String> {
        Err("".into())
    }

}