use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use pb::c2;

pub trait Agent: Send + Sync {
    fn get_config(&self) -> Result<BTreeMap<String, String>, String>;
    fn get_transport(&self) -> Result<String, String>;
    fn set_transport(&self, transport: String) -> Result<(), String>;
    fn list_transports(&self) -> Result<Vec<String>, String>;
    fn get_callback_interval(&self) -> Result<u64, String>;
    fn set_callback_interval(&self, interval: u64) -> Result<(), String>;
    fn list_tasks(&self) -> Result<Vec<c2::Task>, String>;
    fn stop_task(&self, task_id: i64) -> Result<(), String>;
    fn set_callback_uri(&self, uri: String) -> Result<(), String>;
    fn list_callback_uris(&self) -> Result<BTreeSet<String>, String>;
    fn get_active_callback_uri(&self) -> Result<String, String>;
    fn get_next_callback_uri(&self) -> Result<String, String>;
    fn add_callback_uri(&self, uri: String) -> Result<(), String>;
    fn remove_callback_uri(&self, uri: String) -> Result<(), String>;
    fn set_active_callback_uri(&self, uri: String) -> Result<(), String>;
}
