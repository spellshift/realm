use super::AssetsLibrary;
use alloc::borrow::Cow;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use anyhow::Result;
use eldritch_agent::Agent;
use eldritch_macros::eldritch_library_impl;
use pb::c2::FetchAssetRequest;
use std::collections::HashSet;
#[allow(unused_imports)]
use chacha20poly1305::{
    aead::{Aead, KeyInit, generic_array::GenericArray},
    XChaCha20Poly1305,
};

pub mod copy_impl;
pub mod list_impl;
pub mod read_binary_impl;
pub mod read_impl;
pub mod encrypted_assets;

pub use encrypted_assets::{EmbeddedAssets, Embedable};

// Trait for arbitrary backends to get and list assets.
pub trait AssetBackend: Send + Sync + 'static {
    fn get(&self, file_path: &str) -> Result<Vec<u8>>;
    fn assets(&self) -> Vec<Cow<'static, str>>;
}

// An AssetBackend that returns nothing
pub struct EmptyAssets;

impl AssetBackend for EmptyAssets {
    fn get(&self, _: &str) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }
    fn assets(&self) -> Vec<Cow<'static, str>> {
        Vec::new()
    }
}

// An AssetBackend that gets assets from an agent
pub struct AgentAssets {
    pub agent: Arc<dyn Agent>,
    pub remote_assets: Vec<String>,
}

impl AgentAssets {
    pub fn new(agent: Arc<dyn Agent>, remote_assets: Vec<String>) -> Self {
        Self {
            agent,
            remote_assets,
        }
    }
}

impl AssetBackend for AgentAssets {
    fn get(&self, name: &str) -> Result<Vec<u8>> {
        if self.remote_assets.iter().any(|s| s == name) {
            let req = FetchAssetRequest {
                name: name.to_string(),
            };
            return self.agent.fetch_asset(req).map_err(|e| anyhow::anyhow!(e));
        }
        return Err(anyhow::anyhow!("asset not found: {}", name));
    }

    fn assets(&self) -> Vec<Cow<'static, str>> {
        self.remote_assets
            .iter()
            .map(|s| Cow::Owned(s.clone()))
            .collect()
    }
}

#[eldritch_library_impl(AssetsLibrary)]
pub struct StdAssetsLibrary {
    // Stores a vector of boxed trait objects for runtime polymorphism.
    backends: Vec<Arc<dyn AssetBackend>>,
    // Stores all asset names collected so far.
    asset_names: HashSet<String>,
}

impl core::fmt::Debug for StdAssetsLibrary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StdAssetsLibrary").finish()
    }
}

impl StdAssetsLibrary {
    /// Initializes an empty library.
    pub fn new() -> Self {
        StdAssetsLibrary {
            backends: Vec::new(),
            asset_names: HashSet::new(),
        }
    }

    /// Adds an AssetBackend to the library.
    /// The order of addition determines the search precedence.
    /// Asset name shadowing is forbidden
    pub fn add(&mut self, backend: Arc<dyn AssetBackend>) -> anyhow::Result<()> {
        // Make a hashset of the new asset names
        let new_assets: HashSet<String> =
            backend.assets().into_iter().map(Cow::into_owned).collect();
        // See if any name overlap with existin assets
        let colliding_names: Vec<&str> = self
            .asset_names
            .intersection(&new_assets)
            .map(String::as_str)
            .collect();

        if colliding_names.len() > 0 {
            let error_message = format!(
                "asset collision detected. The following asset names already exist in the library: {}",
                colliding_names.join(", ")
            );
            return Err(anyhow::Error::msg(error_message));
        };
        // Box the concrete type and store it as a trait object.
        self.asset_names.extend(new_assets);
        self.backends.push(backend);
        Ok(())
    }

    /// Adds an AssetBackend to the library.
    /// The order of addition determines the search precedence.
    /// Asset name shadowing is allowed
    pub fn add_shadow(&mut self, backend: Arc<dyn AssetBackend>) {
        let assets = backend.assets();
        // This converts the Cow to an owned String only if it isn't already owned.
        self.asset_names
            .extend(assets.iter().map(|c| c.to_string()));
        self.backends.push(backend);
    }
}

impl AssetsLibrary for StdAssetsLibrary {
    fn read_binary(&self, name: String) -> Result<Vec<u8>, String> {
        self.read_binary_impl(&name).map_err(|e| e.to_string())
    }

    fn read(&self, name: String) -> Result<String, String> {
        self.read_impl(name)
    }

    fn copy(&self, src: String, dest: String) -> Result<(), String> {
        self.copy_impl(src, dest)
    }

    fn list(&self) -> Result<Vec<String>, String> {
        self.list_impl()
    }
}
