use super::AgentLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_core::{Interpreter, Value};
use eldritch_macros::eldritch_library_impl;

use crate::TaskWrapper;

#[cfg(feature = "stdlib")]
use eldritch_agent::Agent;

// We need manual Debug impl, and we need to put the macro on the struct.
#[eldritch_library_impl(AgentLibrary)]
pub struct StdAgentLibrary {
    pub agent: Arc<dyn Agent>,
}

impl core::fmt::Debug for StdAgentLibrary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StdAgentLibrary").finish()
    }
}

impl StdAgentLibrary {
    // Removed task_id as it is unused in configuration-only agent lib
    pub fn new(agent: Arc<dyn Agent>) -> Self {
        Self { agent }
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

    fn _terminate_this_process_clowntown(&self) -> Result<(), String> {
        ::std::process::exit(0);
    }

    fn set_callback_interval(&self, interval: i64) -> Result<(), String> {
        self.agent.set_callback_interval(interval as u64)
    }

    // Agent Configuration
    fn get_transport(&self) -> Result<String, String> {
        self.agent.get_transport()
    }

    fn list_transports(&self) -> Result<Vec<String>, String> {
        self.agent.list_transports()
    }
    fn set_active_callback_uri(&self, uri: String) -> Result<(), String> {
        self.agent.set_active_callback_uri(uri)
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
}
