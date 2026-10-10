use alloc::string::String;
use alloc::sync::Arc;

use crate::agent::Agent;

pub fn reset_transport(agent: Arc<dyn Agent>) -> Result<(), String> {
    agent.reset_transport()
}
