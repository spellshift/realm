use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

#[cfg(feature = "stdlib")]
use crate::agent::Agent;
#[cfg(feature = "stdlib")]
use pb::c2;

pub fn fetch_asset(agent: Arc<dyn Agent>, jwt: String, name: String) -> Result<Vec<u8>, String> {
    let req = c2::FetchAssetRequest { name, jwt };
    agent.fetch_asset(req)
}
