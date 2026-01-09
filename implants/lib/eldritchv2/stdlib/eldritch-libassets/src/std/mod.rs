use super::AssetsLibrary;
use crate::RustEmbed;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::marker::PhantomData;
use eldritch_agent::Agent;
use eldritch_macros::eldritch_library_impl;

pub mod copy_impl;
pub mod list_impl;
pub mod read_binary_impl;
pub mod read_impl;

pub struct EmptyAssets;

impl crate::RustEmbed for EmptyAssets {
    fn get(_: &str) -> Option<rust_embed::EmbeddedFile> {
        None
    }
    fn iter() -> impl Iterator<Item = alloc::borrow::Cow<'static, str>> {
        alloc::vec::Vec::<alloc::string::String>::new()
            .into_iter()
            .map(alloc::borrow::Cow::from)
    }
}

#[eldritch_library_impl(AssetsLibrary)]
pub struct StdAssetsLibrary<A: RustEmbed + Send + Sync + 'static> {
    pub agent: Arc<dyn Agent>,
    pub remote_assets: Vec<String>,
    _phantom: PhantomData<A>,
}

impl<A: RustEmbed + Send + Sync + 'static> core::fmt::Debug for StdAssetsLibrary<A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let assets: Vec<_> = A::iter().collect();
        f.debug_struct("StdAssetsLibrary")
            .field("remote_assets", &self.remote_assets)
            .field("embedded_assets", &assets)
            .finish()
    }
}

impl<A: RustEmbed + Send + Sync + 'static> StdAssetsLibrary<A> {
    pub fn new(agent: Arc<dyn Agent>, remote_assets: Vec<String>) -> Self {
        Self {
            agent,
            remote_assets,
            _phantom: PhantomData,
        }
    }
}

impl<A: RustEmbed + Send + Sync + 'static> AssetsLibrary for StdAssetsLibrary<A> {
    fn read_binary(&self, name: String) -> Result<Vec<u8>, String> {
        read_binary_impl::read_binary::<A>(self.agent.clone(), &self.remote_assets, name)
    }

    fn read(&self, name: String) -> Result<String, String> {
        read_impl::read::<A>(self.agent.clone(), &self.remote_assets, name)
    }

    fn copy(&self, src: String, dest: String) -> Result<(), String> {
        copy_impl::copy::<A>(self.agent.clone(), &self.remote_assets, src, dest)
    }

    fn list(&self) -> Result<Vec<String>, String> {
        list_impl::list::<A>(&self.remote_assets)
    }
}
