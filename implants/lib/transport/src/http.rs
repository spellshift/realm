use crate::Transport;


#[derive(Debug, Clone)]
pub struct HTTP {
    http_client: Option<hyper::client::conn::http1::Builder>
}

impl Transport for HTTP {
    fn init() -> Self {
        HTTP{ http_client: None }
    }
    fn new(callback: String, proxy_uri: Option<String>) -> Result<Self> {
        // TODO: setup connection/client hook in proxy, anything else needed
        // before fuctions get called.
        Err(anyhow!("Unimplemented!"))
    }
    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        // TODO: How you wish to handle the `claim_tasks` method.
        Err(anyhow!("Unimplemented!"))
    }
    async fn fetch_asset(
        &mut self,
        request: FetchAssetRequest,
        tx: std::sync::mpsc::Sender<FetchAssetResponse>,
    ) -> Result<()> {
        // TODO: How you wish to handle the `fetch_asset` method.
        Err(anyhow!("Unimplemented!"))
    }
    async fn report_credential(
        &mut self,
        request: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse> {
        // TODO: How you wish to handle the `report_credential` method.
        Err(anyhow!("Unimplemented!"))
    }
    async fn report_file(
        &mut self,
        request: std::sync::mpsc::Receiver<ReportFileRequest>,
    ) -> Result<ReportFileResponse> {
        // TODO: How you wish to handle the `report_file` method.
        Err(anyhow!("Unimplemented!"))
    }
    async fn report_process_list(
        &mut self,
        request: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        // TODO: How you wish to handle the `report_process_list` method.
        Err(anyhow!("Unimplemented!"))
    }
    async fn report_task_output(
        &mut self,
        request: ReportTaskOutputRequest,
    ) -> Result<ReportTaskOutputResponse> {
        // TODO: How you wish to handle the `report_task_output` method.
        Err(anyhow!("Unimplemented!"))
    }
    async fn reverse_shell(
        &mut self,
        rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()> {
        // TODO: How you wish to handle the `reverse_shell` method.
        Err(anyhow!("Unimplemented!"))
    }
}
