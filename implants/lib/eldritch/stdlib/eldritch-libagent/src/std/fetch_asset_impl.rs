use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use eldritch_agent::Context;
use pb::c2::fetch_asset_request;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;
#[cfg(feature = "stdlib")]
use pb::c2;

pub fn fetch_asset(
    agent: Arc<dyn Agent>,
    context: Context,
    name: String,
) -> Result<Vec<u8>, String> {
    let context_val = match context {
        Context::Task(tc) => Some(fetch_asset_request::Context::TaskContext(tc)),
        Context::ShellTask(stc) => Some(fetch_asset_request::Context::ShellTaskContext(stc)),
    };

    let req = c2::FetchAssetRequest {
        name,
        context: context_val,
    };
    agent.fetch_asset(req)
}
