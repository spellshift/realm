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

#[eldritch_library("agent")]
/// The `agent` library provides capabilities for interacting with the agent's internal state and configuration.
pub trait AgentLibrary {
    #[eldritch_method]
    fn get_config(&self) -> Result<BTreeMap<String, Value>, String>;

    #[eldritch_method]
    fn get_id(&self) -> Result<String, String>;

    #[eldritch_method]
    fn get_platform(&self) -> Result<String, String>;

    #[eldritch_method]
    fn _terminate_this_process_clowntown(&self) -> Result<(), String>;

    #[eldritch_method]
    fn set_config(&self, config: BTreeMap<String, Value>) -> Result<(), String>;

    #[eldritch_method]
    fn set_callback_uri(&self, uri: String) -> Result<(), String>;

    // Agent Configuration
    #[eldritch_method]
    fn get_transport(&self) -> Result<String, String>;

    #[eldritch_method]
    fn set_transport(&self, transport: String) -> Result<(), String>;

    #[eldritch_method]
    fn list_transports(&self) -> Result<Vec<String>, String>;

    #[eldritch_method]
    fn get_callback_interval(&self) -> Result<i64, String>;

    #[eldritch_method]
    fn set_callback_interval(&self, interval: i64) -> Result<(), String>;

    // Task Management
    #[eldritch_method]
    fn list_tasks(&self) -> Result<Vec<TaskWrapper>, String>;

    #[eldritch_method]
    fn stop_task(&self, task_id: i64) -> Result<(), String>;

    #[eldritch_method]
    fn eval(&self, interp: &mut Interpreter, code: String) -> Result<Value, String>;
}
