use super::AgentLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::{Interpreter, Value};
use eldritch_macros::eldritch_library_impl;

#[cfg(feature = "stdlib")]
use super::conversion::*;
#[cfg(not(feature = "stdlib"))]
use super::conversion_fake::*;

#[derive(Debug, Default)]
#[eldritch_library_impl(AgentLibrary)]
pub struct AgentLibraryFake;

impl AgentLibrary for AgentLibraryFake {
    fn get_config(&self) -> Result<BTreeMap<String, Value>, String> {
        Ok(BTreeMap::new())
    }

    fn get_id(&self) -> Result<String, String> {
        Ok("fake-id".into())
    }

    fn get_platform(&self) -> Result<String, String> {
        Ok("linux".into())
    }

    fn _terminate_this_process_clowntown(&self) -> Result<(), String> {
        Ok(())
    }

    fn set_config(&self, _config: BTreeMap<String, Value>) -> Result<(), String> {
        Ok(())
    }

    fn set_callback_interval(&self, _interval: i64) -> Result<(), String> {
        Ok(())
    }

    fn set_callback_uri(&self, _uri: String) -> Result<(), String> {
        Ok(())
    }

    fn get_transport(&self) -> Result<String, String> {
        Ok("http".into())
    }

    fn set_transport(&self, _transport: String) -> Result<(), String> {
        Ok(())
    }

    fn list_transports(&self) -> Result<Vec<String>, String> {
        Ok(alloc::vec!["http".into()])
    }

    fn get_callback_interval(&self) -> Result<i64, String> {
        Ok(10)
    }

    fn list_tasks(&self) -> Result<Vec<TaskWrapper>, String> {
        Ok(Vec::new())
    }

    fn stop_task(&self, _task_id: i64) -> Result<(), String> {
        Ok(())
    }

    fn eval(&self, interp: &mut Interpreter, code: String) -> Result<Value, String> {
        interp.interpret(&code)
    }
}

#[cfg(feature = "stdlib")]
use super::agent::Agent;
#[cfg(feature = "stdlib")]
use pb::c2;
#[cfg(feature = "stdlib")]
use transport::SyncTransport;

#[cfg(feature = "stdlib")]
#[derive(Debug, Default)]
pub struct AgentFake;

use alloc::collections::BTreeSet;

#[cfg(feature = "stdlib")]
impl Agent for AgentFake {
    fn get_transport(&self) -> Result<String, String> {
        Ok("http".into())
    }
    fn set_transport(&self, _transport: String) -> Result<(), String> {
        Ok(())
    }
    fn list_transports(&self) -> Result<Vec<String>, String> {
        Ok(alloc::vec!["http".into()])
    }
    fn get_callback_interval(&self) -> Result<u64, String> {
        Ok(10)
    }
    fn set_callback_interval(&self, _interval: u64) -> Result<(), String> {
        Ok(())
    }
    fn set_callback_uri(&self, _uri: String) -> Result<(), String> {
        Ok(())
    }
    fn list_tasks(&self) -> Result<Vec<c2::Task>, String> {
        Ok(Vec::new())
    }
    fn stop_task(&self, _task_id: i64) -> Result<(), String> {
        Ok(())
    }

    fn get_config(&self) -> Result<BTreeMap<String, String>, String> {
        let mut map = BTreeMap::new();
        map.insert("test".to_string(), "config".to_string());
        Ok(map)
    }

    fn list_callback_uris(&self) -> Result<std::collections::BTreeSet<String>, String> {
        Ok(BTreeSet::new())
    }

    fn get_active_callback_uri(&self) -> Result<String, String> {
        Ok(String::new())
    }

    fn get_next_callback_uri(&self) -> Result<String, String> {
        Ok(String::new())
    }

    fn add_callback_uri(&self, _uri: String) -> Result<(), String> {
        Ok(())
    }

    fn remove_callback_uri(&self, _uri: String) -> Result<(), String> {
        Ok(())
    }

    fn set_active_callback_uri(&self, _uri: String) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(feature = "stdlib")]
impl SyncTransport for AgentFake {
    fn fetch_asset(&self, _req: c2::FetchAssetRequest) -> anyhow::Result<Vec<u8>> {
        Ok(Vec::new())
    }
    fn report_credential(
        &self,
        _req: c2::ReportCredentialRequest,
    ) -> anyhow::Result<c2::ReportCredentialResponse> {
        Ok(c2::ReportCredentialResponse::default())
    }
    fn report_file(
        &self,
        _req: c2::ReportFileRequest,
    ) -> anyhow::Result<c2::ReportFileResponse> {
        Ok(c2::ReportFileResponse::default())
    }
    fn report_process_list(
        &self,
        _req: c2::ReportProcessListRequest,
    ) -> anyhow::Result<c2::ReportProcessListResponse> {
        Ok(c2::ReportProcessListResponse::default())
    }
    fn report_task_output(
        &self,
        _req: c2::ReportTaskOutputRequest,
    ) -> anyhow::Result<c2::ReportTaskOutputResponse> {
        Ok(c2::ReportTaskOutputResponse::default())
    }
    fn reverse_shell(
        &self,
        _rx: std::sync::mpsc::Receiver<c2::ReverseShellRequest>,
        _tx: std::sync::mpsc::Sender<c2::ReverseShellResponse>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    fn claim_tasks(
        &self,
        _req: c2::ClaimTasksRequest,
    ) -> anyhow::Result<c2::ClaimTasksResponse> {
        Ok(c2::ClaimTasksResponse::default())
    }
}
