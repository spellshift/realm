use alloc::string::String;
use alloc::sync::Arc;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;

pub fn get_callback_interval(agent: Arc<dyn Agent>) -> Result<i64, String> {
    agent.get_callback_interval().map(|i| i as i64)
}
