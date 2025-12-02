
use eldritch_core::Value;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;
use super::AgentLibrary;

#[cfg(feature = "stdlib")]
use super::conversion::*;

#[derive(Default, Debug)]
#[eldritch_library_impl(AgentLibrary)]
pub struct AgentLibraryFake;

impl AgentLibrary for AgentLibraryFake {
    fn get_config(&self) -> Result<BTreeMap<String, Value>, String> {
        Ok(BTreeMap::new())
    }

    fn get_id(&self) -> Result<String, String> {
        Ok(String::from("fake-agent-uuid"))
    }

    fn get_platform(&self) -> Result<String, String> {
        Ok(String::from("linux"))
    }

    fn kill(&self) -> Result<(), String> {
        Ok(())
    }

    fn set_config(&self, _config: BTreeMap<String, Value>) -> Result<(), String> {
        Ok(())
    }

    fn sleep(&self, _secs: i64) -> Result<(), String> {
        Ok(())
    }

    #[cfg(feature = "stdlib")]
    fn fetch_asset(&self, _name: String) -> Result<Vec<u8>, String> {
        Ok(Vec::new())
    }

    #[cfg(feature = "stdlib")]
    fn report_credential(&self, _credential: CredentialWrapper) -> Result<(), String> {
        Ok(())
    }

    #[cfg(feature = "stdlib")]
    fn report_file(&self, _file: FileWrapper) -> Result<(), String> {
        Ok(())
    }

    #[cfg(feature = "stdlib")]
    fn report_process_list(&self, _list: ProcessListWrapper) -> Result<(), String> {
        Ok(())
    }

    #[cfg(feature = "stdlib")]
    fn report_task_output(&self, _output: String, _error: Option<String>) -> Result<(), String> {
        Ok(())
    }

    #[cfg(feature = "stdlib")]
    fn reverse_shell(&self) -> Result<(), String> {
        Ok(())
    }

    #[cfg(feature = "stdlib")]
    fn claim_tasks(&self) -> Result<Vec<TaskWrapper>, String> {
        Ok(Vec::new())
    }

    #[cfg(feature = "stdlib")]
    fn get_transport(&self) -> Result<String, String> {
        Ok("fake-transport".to_string())
    }

    #[cfg(feature = "stdlib")]
    fn set_transport(&self, _transport: String) -> Result<(), String> {
        Ok(())
    }

    #[cfg(feature = "stdlib")]
    fn add_transport(&self, _transport: String, _config: String) -> Result<(), String> {
        Ok(())
    }

    #[cfg(feature = "stdlib")]
    fn list_transports(&self) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }

    #[cfg(feature = "stdlib")]
    fn get_callback_interval(&self) -> Result<i64, String> {
        Ok(5)
    }

    #[cfg(feature = "stdlib")]
    fn set_callback_interval(&self, _interval: i64) -> Result<(), String> {
        Ok(())
    }

    #[cfg(feature = "stdlib")]
    fn list_tasks(&self) -> Result<Vec<TaskWrapper>, String> {
        Ok(Vec::new())
    }

    #[cfg(feature = "stdlib")]
    fn stop_task(&self, _task_id: i64) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_agent_fake() {
        let agent = AgentLibraryFake::default();
        assert_eq!(agent.get_id().unwrap(), "fake-agent-uuid");
    }
}
