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
    pub report_credential_calls: Arc<Mutex<Vec<c2::ReportCredentialRequest>>>,
    pub report_file_calls: Arc<Mutex<Vec<c2::ReportFileRequest>>>,
    pub report_output_calls: Arc<Mutex<Vec<c2::ReportOutputRequest>>>,
    pub claim_tasks_calls: Arc<Mutex<Vec<c2::ClaimTasksRequest>>>,
    pub stop_task_calls: Arc<Mutex<Vec<i64>>>,
    pub tasks: Arc<Mutex<Vec<c2::Task>>>,
    pub transport: Arc<RwLock<String>>,
    pub set_callback_uri_calls: Arc<Mutex<Vec<String>>>,
    pub reset_transport_calls: Arc<Mutex<usize>>,
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
            report_credential_calls: Arc::new(Mutex::new(Vec::new())),
            report_file_calls: Arc::new(Mutex::new(Vec::new())),
            report_output_calls: Arc::new(Mutex::new(Vec::new())),
            claim_tasks_calls: Arc::new(Mutex::new(Vec::new())),
            stop_task_calls: Arc::new(Mutex::new(Vec::new())),
            tasks: Arc::new(Mutex::new(Vec::new())),
            transport: Arc::new(RwLock::new("http".to_string())),
            set_callback_uri_calls: Arc::new(Mutex::new(Vec::new())),
            reset_transport_calls: Arc::new(Mutex::new(0)),
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

#[async_trait::async_trait]
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
        req: c2::ReportCredentialRequest,
    ) -> Result<c2::ReportCredentialResponse, String> {
        self.report_credential_calls.lock().unwrap().push(req);
        Ok(c2::ReportCredentialResponse::default())
    }

    fn report_file(
        &self,
        req: std::sync::mpsc::Receiver<c2::ReportFileRequest>,
    ) -> Result<c2::ReportFileResponse, String> {
        let mut calls = self.report_file_calls.lock().unwrap();
        for r in req {
            calls.push(r);
        }
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
        req: c2::ReportOutputRequest,
    ) -> Result<c2::ReportOutputResponse, String> {
        self.report_output_calls.lock().unwrap().push(req);
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

    fn claim_tasks(&self, req: c2::ClaimTasksRequest) -> Result<c2::ClaimTasksResponse, String> {
        self.claim_tasks_calls.lock().unwrap().push(req);
        Ok(c2::ClaimTasksResponse {
            tasks: self.tasks.lock().unwrap().clone(),
            shell_tasks: Vec::new(),
        })
    }

    fn get_transport(&self) -> Result<String, String> {
        Ok(self.transport.read().unwrap().clone())
    }

    fn set_transport(&self, transport: String) -> Result<(), String> {
        *self.transport.write().unwrap() = transport;
        Ok(())
    }

    fn reset_transport(&self) -> Result<(), String> {
        *self.reset_transport_calls.lock().unwrap() += 1;
        Ok(())
    }

    fn list_transports(&self) -> Result<Vec<String>, String> {
        Ok(alloc::vec!["http".to_string(), "dns".to_string()])
    }

    fn get_callback_interval(&self) -> Result<u64, String> {
        Ok(10)
    }

    fn set_callback_uri(&self, uri: String) -> Result<(), String> {
        self.set_callback_uri_calls.lock().unwrap().push(uri);
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
        Ok(self.tasks.lock().unwrap().clone())
    }

    fn stop_task(&self, task_id: i64) -> Result<(), String> {
        self.stop_task_calls.lock().unwrap().push(task_id);
        Ok(())
    }

    async fn forward_raw(
        &self,
        _path: String,
        _rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
        _tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    ) -> Result<(), String> {
        Ok(())
    }
}
