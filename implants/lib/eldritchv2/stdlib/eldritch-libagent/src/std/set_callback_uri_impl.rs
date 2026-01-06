use alloc::string::String;
use alloc::sync::Arc;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;

pub fn set_callback_uri(agent: Arc<dyn Agent>, uri: String) -> Result<(), String> {
    agent.set_callback_uri(uri)
}
