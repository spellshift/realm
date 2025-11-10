use crate::Transport;
use anyhow::Result;
use pb::c2::*;
use std::sync::mpsc::{Receiver, Sender};

static CLAIM_TASKS_PATH: &str = "/c2.C2/ClaimTasks";
static FETCH_ASSET_PATH: &str = "/c2.C2/FetchAsset";
static REPORT_CREDENTIAL_PATH: &str = "/c2.C2/ReportCredential";
static REPORT_FILE_PATH: &str = "/c2.C2/ReportFile";
static REPORT_PROCESS_LIST_PATH: &str = "/c2.C2/ReportProcessList";
static REPORT_TASK_OUTPUT_PATH: &str = "/c2.C2/ReportTaskOutput";
static REVERSE_SHELL_PATH: &str = "/c2.C2/ReverseShell";

#[derive(Debug, Clone)]
pub struct HTTP {
    http: Option<tonic::client::Grpc<tonic::transport::Channel>>,
}

impl Transport for HTTP {
    fn init() -> Self {
        HTTP { http: None }
    }

    fn new(callback: String, proxy_uri: Option<String>) -> Result<Self> {
        Ok(Self { http: None })
    }

    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        unimplemented!("todo")
    }

    async fn fetch_asset(
        &mut self,
        request: FetchAssetRequest,
        tx: Sender<FetchAssetResponse>,
    ) -> Result<()> {
        unimplemented!("todo")
    }

    async fn report_credential(
        &mut self,
        request: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse> {
        unimplemented!("todo")
    }

    async fn report_file(
        &mut self,
        request: Receiver<ReportFileRequest>,
    ) -> Result<ReportFileResponse> {
        unimplemented!("todo")
    }

    async fn report_process_list(
        &mut self,
        request: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        unimplemented!("todo")
    }

    async fn report_task_output(
        &mut self,
        request: ReportTaskOutputRequest,
    ) -> Result<ReportTaskOutputResponse> {
        unimplemented!("todo")
    }

    async fn reverse_shell(
        &mut self,
        rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()> {
        unimplemented!("todo")
    }
}
