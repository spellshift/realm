use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use eldritch_agent::{Agent, Context};
use pb::c2;
use pb::eldritch::Process;
use std::sync::{Arc, Mutex, RwLock};

extern crate alloc;

pub struct MockAgent {
    pub config: Arc<RwLock<BTreeMap<String, String>>>,
    pub assets: Arc<Mutex<BTreeMap<String, Vec<u8>>>>,
    pub should_fail_fetch: AtomicBool,
    pub reported_processes: Arc<Mutex<Vec<Process>>>,
    pub start_calls: Arc<Mutex<Vec<(i64, Option<String>)>>>,
    pub repl_calls: Arc<Mutex<Vec<i64>>>,
}

impl MockAgent {
    pub fn new() -> Self {
        let mut map = BTreeMap::new();
        map.insert("key".to_string(), "value".to_string());
        map.insert("interval".to_string(), "5".to_string());
        Self {
            config: Arc::new(RwLock::new(map)),
            assets: Arc::new(Mutex::new(BTreeMap::new())),
            should_fail_fetch: AtomicBool::new(false),
            reported_processes: Arc::new(Mutex::new(Vec::new())),
            start_calls: Arc::new(Mutex::new(Vec::new())),
            repl_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_asset(self, name: &str, content: &[u8]) -> Self {
        self.assets
            .lock()
            .unwrap()
            .insert(name.to_string(), content.to_vec());
        self
    }

    pub fn should_fail(self) -> Self {
        self.should_fail_fetch.store(true, Ordering::SeqCst);
        self
    }
}

impl Default for MockAgent {
    fn default() -> Self {
        Self::new()
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

    fn fetch_asset(&self, req: c2::FetchAssetRequest) -> Result<Vec<u8>, String> {
        if self.should_fail_fetch.load(Ordering::SeqCst) {
            return Err("Failed to fetch asset".to_string());
        }
        if let Some(data) = self.assets.lock().unwrap().get(&req.name) {
            Ok(data.clone())
        } else {
            Err("Asset not found".to_string())
        }
    }

    fn report_credential(
        &self,
        _req: c2::ReportCredentialRequest,
    ) -> Result<c2::ReportCredentialResponse, String> {
        Ok(c2::ReportCredentialResponse::default())
    }

    fn report_file(
        &self,
        _req: std::sync::mpsc::Receiver<c2::ReportFileRequest>,
    ) -> Result<c2::ReportFileResponse, String> {
        Ok(c2::ReportFileResponse::default())
    }

    fn report_process_list(
        &self,
        req: c2::ReportProcessListRequest,
    ) -> Result<c2::ReportProcessListResponse, String> {
        if let Some(list) = req.list {
            if list.list.is_empty() {
                return Err("Process list is empty".to_string());
            }
            for p in &list.list {
                if p.principal.is_empty() {
                    return Err(alloc::format!(
                        "Principal is empty for process pid={}",
                        p.pid
                    ));
                }
            }
            self.reported_processes.lock().unwrap().extend(list.list);
        } else {
            return Err("No process list received".to_string());
        }
        Ok(c2::ReportProcessListResponse::default())
    }

    fn report_output(
        &self,
        _req: c2::ReportOutputRequest,
    ) -> Result<c2::ReportOutputResponse, String> {
        Ok(c2::ReportOutputResponse::default())
    }

    fn create_portal(&self, _context: Context) -> Result<(), String> {
        Ok(())
    }

    fn start_reverse_shell(&self, context: Context, cmd: Option<String>) -> Result<(), String> {
        self.start_calls.lock().unwrap().push((
            match context {
                Context::Task(t) => t.task_id,
                Context::ShellTask(s) => s.shell_task_id,
            },
            cmd,
        ));
        Ok(())
    }

    fn start_repl_reverse_shell(&self, context: Context) -> Result<(), String> {
        self.repl_calls.lock().unwrap().push(match context {
            Context::Task(t) => t.task_id,
            Context::ShellTask(s) => s.shell_task_id,
        });
        Ok(())
    }

    fn claim_tasks(&self, _req: c2::ClaimTasksRequest) -> Result<c2::ClaimTasksResponse, String> {
        Ok(c2::ClaimTasksResponse::default())
    }

    fn get_transport(&self) -> Result<String, String> {
        Ok("http".to_string())
    }

    fn set_transport(&self, _transport: String) -> Result<(), String> {
        Ok(())
    }

    fn list_transports(&self) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }

    fn get_callback_interval(&self) -> Result<u64, String> {
        Ok(10)
    }

    fn set_callback_uri(&self, _uri: String) -> Result<(), String> {
        Ok(())
    }

    fn list_callback_uris(&self) -> Result<BTreeSet<String>, String> {
        Ok(BTreeSet::new())
    }

    fn get_active_callback_uri(&self) -> Result<String, String> {
        Ok(String::new())
    }

    fn get_next_callback_uri(&self) -> Result<String, String> {
        Ok(String::new())
    }

    fn add_callback_uri(&self, _uri: String) -> Result<(), String> {
        Ok(())
    }

    fn remove_callback_uri(&self, _uri: String) -> Result<(), String> {
        Ok(())
    }

    fn list_tasks(&self) -> Result<Vec<c2::Task>, String> {
        Ok(Vec::new())
    }

    fn stop_task(&self, _task_id: i64) -> Result<(), String> {
        Ok(())
    }
}
