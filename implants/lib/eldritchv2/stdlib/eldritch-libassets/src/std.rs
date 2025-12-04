use super::AssetsLibrary;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use anyhow::Result;
use eldritch_macros::eldritch_library_impl;
use eldritch_libagent::agent::Agent;
use pb::c2::FetchAssetRequest;
use rust_embed::RustEmbed;
use std::io::Write;

#[cfg(debug_assertions)]
#[derive(RustEmbed)]
#[folder = "../../../../../bin/embedded_files_test"]
pub struct Asset;

#[cfg(not(feature = "imix"))]
#[cfg(not(debug_assertions))]
#[derive(RustEmbed)]
#[folder = "../../../../../implants/golem/embed_files_golem_prod"]
pub struct Asset;

#[cfg(feature = "imix")]
#[cfg(not(debug_assertions))]
#[derive(RustEmbed)]
#[folder = "../../../../../implants/imix/install_scripts"]
pub struct Asset;

#[eldritch_library_impl(AssetsLibrary)]
pub struct StdAssetsLibrary {
    pub agent: Arc<dyn Agent>,
    pub remote_assets: Vec<String>,
}

impl core::fmt::Debug for StdAssetsLibrary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StdAssetsLibrary")
         .field("remote_assets", &self.remote_assets)
         .finish()
    }
}

impl StdAssetsLibrary {
    pub fn new(agent: Arc<dyn Agent>, remote_assets: Vec<String>) -> Self {
        Self { agent, remote_assets }
    }

    fn read_binary_embedded(&self, src: &str) -> Result<Vec<u8>> {
        let src_file_bytes = match Asset::get(src) {
            Some(local_src_file) => local_src_file.data,
            None => return Err(anyhow::anyhow!("Embedded file {src} not found.")),
        };
        Ok(src_file_bytes.to_vec())
    }

    fn _read_binary(&self, name: &str) -> Result<Vec<u8>> {
        if self.remote_assets.iter().any(|s| s == name) {
            let req = FetchAssetRequest { name: name.to_string() };
            return self.agent.fetch_asset(req).map_err(|e| anyhow::anyhow!(e));
        }
        self.read_binary_embedded(name)
    }
}

impl AssetsLibrary for StdAssetsLibrary {
    fn read_binary(&self, name: String) -> Result<Vec<u8>, String> {
        self._read_binary(&name).map_err(|e| e.to_string())
    }

    fn read(&self, name: String) -> Result<String, String> {
        let bytes = self._read_binary(&name).map_err(|e| e.to_string())?;
        String::from_utf8(bytes).map_err(|e| e.to_string())
    }

    fn copy(&self, src: String, dest: String) -> Result<(), String> {
        let bytes = self._read_binary(&src).map_err(|e| e.to_string())?;
        let mut file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
        file.write_all(&bytes).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn list(&self) -> Result<Vec<String>, String> {
        let mut files: Vec<String> = Asset::iter().map(|f| f.as_ref().to_string()).collect();
        // Append remote assets to the list if they are not already there
        for remote in &self.remote_assets {
             if !files.contains(remote) {
                 files.push(remote.clone());
             }
        }
        Ok(files)
    }
}
