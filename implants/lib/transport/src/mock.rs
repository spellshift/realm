use anyhow::Result;
use pb::c2::*;
use std::sync::mpsc::{Receiver, Sender};

use mockall::{mock, predicate::*};

mock! {
    pub Transport {}
    impl Clone for Transport {
        fn clone(&self) -> Self;
    }
    impl super::Transport for Transport {
        fn init() -> Self;

        fn new(uri: String, config: pb::config::Config) -> Result<Self>;

        async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse>;

        async fn fetch_asset(
            &mut self,
            request: FetchAssetRequest,
            sender: Sender<FetchAssetResponse>,
        ) -> Result<()>;

        async fn report_credential(
            &mut self,
            request: ReportCredentialRequest,
        ) -> Result<ReportCredentialResponse>;

        async fn report_file(
            &mut self,
            request: Receiver<ReportFileRequest>,
        ) -> Result<ReportFileResponse>;

        async fn report_process_list(
            &mut self,
            request: ReportProcessListRequest,
        ) -> Result<ReportProcessListResponse>;

        async fn report_task_output(
            &mut self,
            request: ReportTaskOutputRequest,
        ) -> Result<ReportTaskOutputResponse>;

        async fn create_portal(
            &mut self,
            rx: tokio::sync::mpsc::Receiver<CreatePortalRequest>,
            tx: tokio::sync::mpsc::Sender<CreatePortalResponse>,
        ) -> Result<()>;

        fn get_type(&mut self) -> pb::c2::active_transport::Type {
            return pb::c2::active_transport::Type::TransportUnspecified;
        }

        fn is_active(&self) -> bool;

        fn name(&self) -> &'static str;

        fn list_available(&self) -> Vec<String>;
    }
}
