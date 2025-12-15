#![no_std]
#![allow(clippy::mutable_key_type)]

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::pin::Pin;
use core::future::Future;

#[cfg(feature = "stdlib")]
use pb::c2;
#[cfg(feature = "stdlib")]
use transport::SyncTransport;

pub type SubtaskFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

pub trait Agent: Send + Sync {
    fn get_config(&self) -> Result<BTreeMap<String, String>, String>;
    fn get_transport(&self) -> Result<String, String>;
    fn set_transport(&self, transport: String) -> Result<(), String>;
    fn list_transports(&self) -> Result<Vec<String>, String>;
    fn get_callback_interval(&self) -> Result<u64, String>;
    fn set_callback_interval(&self, interval: u64) -> Result<(), String>;
    #[cfg(feature = "stdlib")]
    fn list_tasks(&self) -> Result<Vec<c2::Task>, String>;
    fn stop_task(&self, task_id: i64) -> Result<(), String>;
    fn set_callback_uri(&self, uri: String) -> Result<(), String>;
    fn list_callback_uris(&self) -> Result<BTreeSet<String>, String>;
    fn get_active_callback_uri(&self) -> Result<String, String>;
    fn get_next_callback_uri(&self) -> Result<String, String>;
    fn add_callback_uri(&self, uri: String) -> Result<(), String>;
    fn remove_callback_uri(&self, uri: String) -> Result<(), String>;
    fn set_active_callback_uri(&self, uri: String) -> Result<(), String>;
    fn spawn_subtask(&self, task_id: i64, name: String, future: SubtaskFuture) -> Result<(), String>;
    #[cfg(feature = "stdlib")]
    fn get_sync_transport(&self) -> Option<Arc<dyn SyncTransport>>;
}
