use alloc::string::String;
use alloc::sync::Arc;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;

pub fn set_callback_interval(agent: Arc<dyn Agent>, interval: i64) -> Result<(), String> {
    agent.set_callback_interval(interval as u64)
}
