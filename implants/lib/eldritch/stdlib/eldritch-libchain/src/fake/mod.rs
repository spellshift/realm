use super::ChainLibrary;

use alloc::string::String;

use eldritch_macros::eldritch_library_impl;

#[eldritch_library_impl(ChainLibrary)]
pub struct FakeChainLibrary;

impl core::fmt::Debug for FakeChainLibrary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FakeChainLibrary").finish()
    }
}

impl FakeChainLibrary {
    pub fn new() -> Self {
        Self
    }
}

impl ChainLibrary for FakeChainLibrary {
    fn tcp(&self, _addr: String) -> Result<i64, String> {
        Err("chain is not supported in this environment".into())
    }
}
