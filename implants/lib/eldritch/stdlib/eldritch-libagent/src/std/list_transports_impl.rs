use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;

pub fn list_transports(agent: Arc<dyn Agent>) -> Result<Vec<String>, String> {
    agent.list_transports()
}
