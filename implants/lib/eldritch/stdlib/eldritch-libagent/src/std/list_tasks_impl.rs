use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::TaskWrapper;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;

pub fn list_tasks(agent: Arc<dyn Agent>) -> Result<Vec<TaskWrapper>, String> {
    let tasks = agent.list_tasks()?;
    Ok(tasks.into_iter().map(TaskWrapper).collect())
}
