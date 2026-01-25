use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::TaskWrapper;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;
#[cfg(feature = "stdlib")]
use pb::c2;

pub fn claim_tasks(agent: Arc<dyn Agent>) -> Result<Vec<TaskWrapper>, String> {
    let req = c2::ClaimTasksRequest { beacon: None };
    let resp = agent.claim_tasks(req)?;
    Ok(resp.tasks.into_iter().map(TaskWrapper).collect())
}
