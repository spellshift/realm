use super::AgentLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;

use crate::{CredentialWrapper, FileWrapper, ProcessListWrapper, TaskWrapper};

#[cfg(feature = "stdlib")]
use crate::agent::Agent;

// Re-export TaskContext from eldritch_agent
pub use eldritch_agent::TaskContext;

pub mod claim_tasks_impl;
pub mod fetch_asset_impl;
pub mod get_callback_interval_impl;
pub mod get_config_impl;
pub mod get_transport_impl;
pub mod list_tasks_impl;
pub mod list_transports_impl;
pub mod report_credential_impl;
pub mod report_file_impl;
pub mod report_process_list_impl;
pub mod report_task_output_impl;
pub mod set_callback_interval_impl;
pub mod set_callback_uri_impl;
pub mod stop_task_impl;
pub mod terminate_impl;

// We need manual Debug impl, and we need to put the macro on the struct.
#[eldritch_library_impl(AgentLibrary)]
pub struct StdAgentLibrary {
    pub agent: Arc<dyn Agent>,
    pub task_context: TaskContext,
}

impl core::fmt::Debug for StdAgentLibrary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StdAgentLibrary")
            .field("task_context", &self.task_context)
            .finish()
    }
}

impl StdAgentLibrary {
    pub fn new(agent: Arc<dyn Agent>, task_context: TaskContext) -> Self {
        Self {
            agent,
            task_context,
        }
    }
}

impl AgentLibrary for StdAgentLibrary {
    fn get_config(&self) -> Result<BTreeMap<String, Value>, String> {
        get_config_impl::get_config(self.agent.clone())
    }

    fn _terminate_this_process_clowntown(&self) -> Result<(), String> {
        terminate_impl::terminate()
    }

    fn set_callback_interval(&self, interval: i64) -> Result<(), String> {
        set_callback_interval_impl::set_callback_interval(self.agent.clone(), interval)
    }

    // Interactivity
    fn fetch_asset(&self, name: String) -> Result<Vec<u8>, String> {
        fetch_asset_impl::fetch_asset(self.agent.clone(), self.task_context.clone(), name)
    }

    fn report_credential(&self, credential: CredentialWrapper) -> Result<(), String> {
        report_credential_impl::report_credential(
            self.agent.clone(),
            self.task_context.clone(),
            credential,
        )
    }

    fn report_file(&self, file: FileWrapper) -> Result<(), String> {
        report_file_impl::report_file(self.agent.clone(), self.task_context.clone(), file)
    }

    fn report_process_list(&self, list: ProcessListWrapper) -> Result<(), String> {
        report_process_list_impl::report_process_list(
            self.agent.clone(),
            self.task_context.clone(),
            list,
        )
    }

    fn report_task_output(&self, output: String, error: Option<String>) -> Result<(), String> {
        report_task_output_impl::report_task_output(
            self.agent.clone(),
            self.task_context.clone(),
            output,
            error,
        )
    }

    fn claim_tasks(&self) -> Result<Vec<TaskWrapper>, String> {
        claim_tasks_impl::claim_tasks(self.agent.clone())
    }

    // Agent Configuration
    fn get_transport(&self) -> Result<String, String> {
        get_transport_impl::get_transport(self.agent.clone())
    }

    fn list_transports(&self) -> Result<Vec<String>, String> {
        list_transports_impl::list_transports(self.agent.clone())
    }

    fn set_callback_uri(&self, uri: String) -> Result<(), String> {
        set_callback_uri_impl::set_callback_uri(self.agent.clone(), uri)
    }

    fn get_callback_interval(&self) -> Result<i64, String> {
        get_callback_interval_impl::get_callback_interval(self.agent.clone())
    }

    // Task Management
    fn list_tasks(&self) -> Result<Vec<TaskWrapper>, String> {
        list_tasks_impl::list_tasks(self.agent.clone())
    }

    fn stop_task(&self, task_id: i64) -> Result<(), String> {
        stop_task_impl::stop_task(self.agent.clone(), task_id)
    }
}
