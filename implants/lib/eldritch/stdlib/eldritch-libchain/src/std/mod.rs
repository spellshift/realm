use super::ChainLibrary;
pub mod chain_impl;

use alloc::string::String;
use alloc::sync::Arc;
use anyhow::Result;
use eldritch_agent::Agent;
use eldritch_macros::eldritch_library_impl;

#[eldritch_library_impl(ChainLibrary)]
pub struct StdChainLibrary {
    pub agent: Arc<dyn Agent>,
}

impl core::fmt::Debug for StdChainLibrary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StdChainLibrary").finish()
    }
}

impl StdChainLibrary {
    pub fn new(agent: Arc<dyn Agent>) -> Self {
        Self { agent }
    }
}

impl ChainLibrary for StdChainLibrary {
    fn tcp(&self, addr: String) -> Result<i64, String> {
        chain_impl::tcp(addr, self.agent.clone()).map_err(|e| e.to_string())
    }
}
