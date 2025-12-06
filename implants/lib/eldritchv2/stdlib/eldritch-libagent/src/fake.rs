use super::AgentLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

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

#[cfg(all(feature = "stdlib", feature = "fake_bindings"))]
pub use self::inner_fake::AgentFake;

#[cfg(all(feature = "stdlib", feature = "fake_bindings"))]
mod inner_fake {
    use super::super::agent::Agent;
    use alloc::collections::BTreeMap;
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;
    use pb::c2;
    use std::sync::{Arc, Mutex};

    #[derive(Default, Debug)]
    pub struct AgentFakeState {
        pub tasks: Vec<c2::Task>,
        pub credentials: Vec<c2::ReportCredentialRequest>,
        pub files: Vec<c2::ReportFileRequest>,
        pub processes: Vec<c2::ReportProcessListRequest>,
        pub task_outputs: Vec<c2::ReportTaskOutputRequest>,
        pub assets: BTreeMap<String, Vec<u8>>,
        pub transports: BTreeMap<String, String>,
        pub callback_interval: u64,
        pub reverse_shell_active: bool,
    }

    #[derive(Clone, Default, Debug)]
    pub struct AgentFake {
        pub state: Arc<Mutex<AgentFakeState>>,
    }

    impl AgentFake {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn add_task(&self, task: c2::Task) {
            let mut state = self.state.lock().unwrap();
            state.tasks.push(task);
        }

        pub fn add_asset(&self, name: String, data: Vec<u8>) {
            let mut state = self.state.lock().unwrap();
            state.assets.insert(name, data);
        }

        pub fn get_reported_credentials(&self) -> Vec<c2::ReportCredentialRequest> {
            self.state.lock().unwrap().credentials.clone()
        }

        pub fn get_reported_files(&self) -> Vec<c2::ReportFileRequest> {
            self.state.lock().unwrap().files.clone()
        }

        pub fn get_reported_processes(&self) -> Vec<c2::ReportProcessListRequest> {
            self.state.lock().unwrap().processes.clone()
        }

        pub fn get_reported_task_outputs(&self) -> Vec<c2::ReportTaskOutputRequest> {
            self.state.lock().unwrap().task_outputs.clone()
        }

        pub fn is_reverse_shell_active(&self) -> bool {
            self.state.lock().unwrap().reverse_shell_active
        }
    }

    impl Agent for AgentFake {
        fn fetch_asset(&self, req: c2::FetchAssetRequest) -> Result<Vec<u8>, String> {
            let state = self.state.lock().unwrap();
            state
                .assets
                .get(&req.name)
                .cloned()
                .ok_or_else(|| "Asset not found".to_string())
        }

        fn report_credential(
            &self,
            req: c2::ReportCredentialRequest,
        ) -> Result<c2::ReportCredentialResponse, String> {
            let mut state = self.state.lock().unwrap();
            state.credentials.push(req);
            Ok(c2::ReportCredentialResponse {})
        }

        fn report_file(
            &self,
            req: c2::ReportFileRequest,
        ) -> Result<c2::ReportFileResponse, String> {
            let mut state = self.state.lock().unwrap();
            state.files.push(req);
            Ok(c2::ReportFileResponse {})
        }

        fn report_process_list(
            &self,
            req: c2::ReportProcessListRequest,
        ) -> Result<c2::ReportProcessListResponse, String> {
            let mut state = self.state.lock().unwrap();
            state.processes.push(req);
            Ok(c2::ReportProcessListResponse {})
        }

        fn report_task_output(
            &self,
            req: c2::ReportTaskOutputRequest,
        ) -> Result<c2::ReportTaskOutputResponse, String> {
            let mut state = self.state.lock().unwrap();
            state.task_outputs.push(req);
            Ok(c2::ReportTaskOutputResponse {})
        }

        fn reverse_shell(&self) -> Result<(), String> {
            let mut state = self.state.lock().unwrap();
            state.reverse_shell_active = !state.reverse_shell_active;
            Ok(())
        }

        fn start_reverse_shell(&self, _task_id: i64, _cmd: Option<String>) -> Result<(), String> {
            let mut state = self.state.lock().unwrap();
            state.reverse_shell_active = true;
            Ok(())
        }

        fn start_repl_reverse_shell(&self, _task_id: i64) -> Result<(), String> {
            let mut state = self.state.lock().unwrap();
            state.reverse_shell_active = true;
            Ok(())
        }

        fn claim_tasks(
            &self,
            _req: c2::ClaimTasksRequest,
        ) -> Result<c2::ClaimTasksResponse, String> {
            let mut state = self.state.lock().unwrap();
            // Move all pending tasks to the response
            let tasks = state.tasks.drain(..).collect();
            Ok(c2::ClaimTasksResponse { tasks })
        }

        fn get_transport(&self) -> Result<String, String> {
            // Default or first? Let's say "http" if not set, or we can check keys
            let state = self.state.lock().unwrap();
            // Just return a dummy or first key
            state
                .transports
                .keys()
                .next()
                .cloned()
                .ok_or_else(|| "No transports".to_string())
        }

        fn set_transport(&self, _transport: String) -> Result<(), String> {
            // For now just ensure it exists? or set current?
            Ok(())
        }

        fn add_transport(&self, transport: String, config: String) -> Result<(), String> {
            let mut state = self.state.lock().unwrap();
            state.transports.insert(transport, config);
            Ok(())
        }

        fn list_transports(&self) -> Result<Vec<String>, String> {
            let state = self.state.lock().unwrap();
            Ok(state.transports.keys().cloned().collect())
        }

        fn get_callback_interval(&self) -> Result<u64, String> {
            let state = self.state.lock().unwrap();
            Ok(state.callback_interval)
        }

        fn set_callback_interval(&self, interval: u64) -> Result<(), String> {
            let mut state = self.state.lock().unwrap();
            state.callback_interval = interval;
            Ok(())
        }

        fn list_tasks(&self) -> Result<Vec<c2::Task>, String> {
            let state = self.state.lock().unwrap();
            Ok(state.tasks.clone())
        }

        fn stop_task(&self, task_id: i64) -> Result<(), String> {
            let mut state = self.state.lock().unwrap();
            state.tasks.retain(|t| t.id != task_id);
            Ok(())
        }
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_agent_fake() {
        let agent = AgentLibraryFake;
        assert_eq!(agent.get_id().unwrap(), "fake-agent-uuid");
    }

    #[cfg(feature = "stdlib")]
    #[test]
    fn test_agent_fake_impl() {
        use super::super::agent::Agent;
        use super::inner_fake::AgentFake;
        use pb::c2;
        use pb::eldritch::Credential;

        let agent = AgentFake::new();

        // Test config
        agent.set_callback_interval(10).unwrap();
        assert_eq!(agent.get_callback_interval().unwrap(), 10);

        // Test tasks
        let task = c2::Task {
            id: 123,
            quest_name: "test_quest".to_string(),
            ..Default::default()
        };
        agent.add_task(task.clone());

        let claimed = agent.claim_tasks(c2::ClaimTasksRequest::default()).unwrap();
        assert_eq!(claimed.tasks.len(), 1);
        assert_eq!(claimed.tasks[0].id, 123);

        // Claim again, should be empty
        let claimed2 = agent.claim_tasks(c2::ClaimTasksRequest::default()).unwrap();
        assert!(claimed2.tasks.is_empty());

        // Test credentials reporting
        let cred = c2::ReportCredentialRequest {
            credential: Some(Credential {
                principal: "user".to_string(),
                secret: "pass".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };
        agent.report_credential(cred.clone()).unwrap();

        let reported = agent.get_reported_credentials();
        assert_eq!(reported.len(), 1);
        assert_eq!(reported[0].credential.as_ref().unwrap().principal, "user");
    }
}
