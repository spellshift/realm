use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use super::TaskContext;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;
#[cfg(feature = "stdlib")]
use pb::c2;

pub fn fetch_asset(
    agent: Arc<dyn Agent>,
    task_context: TaskContext,
    name: String,
) -> Result<Vec<u8>, String> {
    let req = c2::FetchAssetRequest {
        name,
        context: Some(task_context.into()),
    };
    agent.fetch_asset(req)
}
