use anyhow::Result;
use api::pb::c2::{
    DownloadFileRequest, DownloadFileResponse, ReportCredentialRequest, ReportCredentialResponse,
    ReportFileRequest, ReportFileResponse, ReportProcessListRequest, ReportProcessListResponse,
    ReportTaskOutputRequest, ReportTaskOutputResponse,
};
use std::sync::mpsc::{Receiver, Sender};

pub trait Transport {
    async fn download_file(
        &mut self,
        request: DownloadFileRequest,
        sender: Sender<DownloadFileResponse>,
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
}
