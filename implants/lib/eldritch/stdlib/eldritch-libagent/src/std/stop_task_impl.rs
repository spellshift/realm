use alloc::string::String;
use alloc::sync::Arc;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;

pub fn stop_task(agent: Arc<dyn Agent>, task_id: i64) -> Result<(), String> {
    agent.stop_task(task_id)
}
