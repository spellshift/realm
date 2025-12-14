#![allow(clippy::mutable_key_type)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use pb::c2;

pub type SubtaskFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

pub trait Agent: Send + Sync {
    /**
     * Callback Management
     */
    fn get_config(&self) -> Result<BTreeMap<String, String>, String>;
    fn get_callback_interval(&self) -> Result<u64, String>;
    fn set_callback_interval(&self, interval: u64) -> Result<(), String>;
    fn list_transports(&self) -> Result<Vec<String>, String>;
    fn get_transport(&self) -> Result<String, String>;
    fn list_callback_uris(&self) -> Result<BTreeSet<String>, String>;
    fn add_callback_uri(&self, uri: String) -> Result<(), String>;
    fn remove_callback_uri(&self, uri: String) -> Result<(), String>;
    fn set_active_callback_uri(&self, uri: String) -> Result<(), String>;
    fn get_active_callback_uri(&self) -> Result<String, String>;
    fn get_next_callback_uri(&self) -> Result<String, String>;

    /**
     * Task Management
     */
    fn list_tasks(&self) -> Result<Vec<c2::Task>, String>;
    fn stop_task(&self, task_id: i64) -> Result<(), String>;
    fn spawn_subtask(
        &self,
        task_id: i64,
        name: String,
        future: SubtaskFuture,
    ) -> Result<(), String>;
}
