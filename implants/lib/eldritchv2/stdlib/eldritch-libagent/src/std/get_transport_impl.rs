use alloc::string::String;
use alloc::sync::Arc;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;

pub fn get_transport(agent: Arc<dyn Agent>) -> Result<String, String> {
    agent.get_transport()
}
