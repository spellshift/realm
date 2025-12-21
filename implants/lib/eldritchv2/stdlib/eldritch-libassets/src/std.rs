use super::AssetsLibrary;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::borrow::Cow;
use anyhow::Result;
use eldritch_agent::Agent;
use rust_embed;
use core::marker::PhantomData;
use eldritch_macros::eldritch_library_impl;
use pb::c2::FetchAssetRequest;
use std::io::Write;
use std::collections::HashSet;


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

// An AssetBackend that gets assets from a rust_embed::Embed
pub struct EmbeddedAssets<T: rust_embed::Embed> {
    _phantom: PhantomData<T>,
}

impl<T: rust_embed::Embed> EmbeddedAssets<T> {
    pub fn new() -> Self { // No arguments needed
        Self { _phantom: PhantomData }
    }
}

impl<T: rust_embed::Embed + Send + Sync + 'static> AssetBackend for EmbeddedAssets<T> {
    fn get(&self, name: &str) -> Result<Vec<u8>> {
        // T::get is a static method from the rust_embed::Embed trait
        T::get(name)
            .map(|file| file.data.to_vec())
            .ok_or_else(|| anyhow::anyhow!("asset not found: {}", name))
    }

    fn assets(&self) -> Vec<Cow<'static, str>> {
        // T::iter() returns an iterator of Cow<'static, str>
        T::iter().collect()
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
        f.debug_struct("StdAssetsLibrary")
            .finish()
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
        let new_assets: HashSet<String> = backend.assets().into_iter()
            .map(Cow::into_owned)
            .collect();
        // See if any name overlap with existin assets
        let colliding_names: Vec<&str> = self.asset_names.intersection(&new_assets)
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
        self.asset_names.extend(
            assets.iter().map(|c| c.to_string())
        );
        self.backends.push(backend);
    }

    fn _read_binary(&self, name: &str) -> Result<Vec<u8>> {
        // We have a hashmap of all the names, might as well use it
        if !self.asset_names.contains(name) {
            return Err(anyhow::anyhow!("asset not found: {}", name));
        };
        // Iterate through the boxed trait objects (maintaining precedence order)
        for backend in &self.backends {
            
            if let Ok(file) = backend.get(&name) {
                // Return immediately upon the first match
                return Ok(file);
            }
        }
        Err(anyhow::anyhow!("asset not found: {}", name))
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
        Ok(self.asset_names.iter().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeMap;
    use alloc::string::ToString;
    use eldritch_agent::{Agent};
    use pb::c2;
    use std::borrow::Cow;
    use std::collections::BTreeSet;
    use std::sync::Mutex;

    use rust_embed::Embed as CrateEmbed;

    #[cfg(debug_assertions)]
    #[derive(rust_embed::Embed)]
    #[folder = "../../../../../bin/embedded_files_test"]
    pub struct TestAsset;

    struct MockAgent {
        assets: Mutex<BTreeMap<String, Vec<u8>>>,
        should_fail_fetch: bool,
    }

    impl MockAgent {
        fn new() -> Self {
            Self {
                assets: Mutex::new(BTreeMap::new()),
                should_fail_fetch: false,
            }
        }

        fn with_asset(self, name: &str, content: &[u8]) -> Self {
            self.assets
                .lock()
                .unwrap()
                .insert(name.to_string(), content.to_vec());
            self
        }

        fn should_fail(mut self) -> Self {
            self.should_fail_fetch = true;
            self
        }
    }

    impl Agent for MockAgent {
        fn fetch_asset(&self, req: c2::FetchAssetRequest) -> Result<Vec<u8>, String> {
            if self.should_fail_fetch {
                return Err("Failed to fetch asset".to_string());
            }
            if let Some(data) = self.assets.lock().unwrap().get(&req.name) {
                Ok(data.clone())
            } else {
                Err("Asset not found".to_string())
            }
        }

        fn report_credential(
            &self,
            _req: c2::ReportCredentialRequest,
        ) -> Result<c2::ReportCredentialResponse, String> {
            Ok(c2::ReportCredentialResponse::default())
        }
        fn report_file(
            &self,
            _req: c2::ReportFileRequest,
        ) -> Result<c2::ReportFileResponse, String> {
            Ok(c2::ReportFileResponse::default())
        }
        fn report_process_list(
            &self,
            _req: c2::ReportProcessListRequest,
        ) -> Result<c2::ReportProcessListResponse, String> {
            Ok(c2::ReportProcessListResponse::default())
        }
        fn report_task_output(
            &self,
            _req: c2::ReportTaskOutputRequest,
        ) -> Result<c2::ReportTaskOutputResponse, String> {
            Ok(c2::ReportTaskOutputResponse::default())
        }
        fn start_reverse_shell(&self, _task_id: i64, _cmd: Option<String>) -> Result<(), String> {
            Ok(())
        }
        fn start_repl_reverse_shell(&self, _task_id: i64) -> Result<(), String> {
            Ok(())
        }
        fn claim_tasks(
            &self,
            _req: c2::ClaimTasksRequest,
        ) -> Result<c2::ClaimTasksResponse, String> {
            Ok(c2::ClaimTasksResponse::default())
        }
        fn get_config(&self) -> Result<BTreeMap<String, String>, String> {
            Ok(BTreeMap::new())
        }
        fn get_transport(&self) -> Result<String, String> {
            Ok("mock".into())
        }
        fn set_transport(&self, _transport: String) -> Result<(), String> {
            Ok(())
        }
        fn list_transports(&self) -> Result<Vec<String>, String> {
            Ok(Vec::new())
        }
        fn get_callback_interval(&self) -> Result<u64, String> {
            Ok(10)
        }
        fn set_callback_interval(&self, _interval: u64) -> Result<(), String> {
            Ok(())
        }
        fn list_tasks(&self) -> Result<Vec<c2::Task>, String> {
            Ok(Vec::new())
        }
        fn stop_task(&self, _task_id: i64) -> Result<(), String> {
            Ok(())
        }
        fn set_callback_uri(&self, _uri: String) -> std::result::Result<(), String> {
            Ok(())
        }
        fn list_callback_uris(
            &self,
        ) -> std::result::Result<std::collections::BTreeSet<String>, String> {
            Ok(BTreeSet::new())
        }
        fn get_active_callback_uri(&self) -> std::result::Result<String, String> {
            Ok(String::new())
        }
        fn get_next_callback_uri(&self) -> std::result::Result<String, String> {
            Ok(String::new())
        }
        fn add_callback_uri(&self, _uri: String) -> std::result::Result<(), String> {
            Ok(())
        }
        fn remove_callback_uri(&self, _uri: String) -> std::result::Result<(), String> {
            Ok(())
        }
        fn set_active_callback_uri(&self, _uri: String) -> std::result::Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn test_read_binary_embedded_success() -> anyhow::Result<()> {
        let mut lib = StdAssetsLibrary::new(); 
        lib.add(Arc::new(EmbeddedAssets::<TestAsset>::new()))?;
        let content = lib.read_binary("print/main.eldritch".to_string());
        assert!(content.is_ok());
        let content = content.unwrap();
        assert!(!content.is_empty());
        assert_eq!(
            std::str::from_utf8(&content).unwrap().trim(),
            "print(\"This script just prints\")"
        );
        Ok(())
    }

    #[test]
    fn test_read_binary_embedded_fail() -> anyhow::Result<()> {
        let mut lib = StdAssetsLibrary::new(); 
        lib.add(Arc::new(EmbeddedAssets::<TestAsset>::new()))?;
        assert!(lib.read_binary("nonexistent_file".to_string()).is_err());
        Ok(())
    }

    #[test]
    fn test_read_binary_remote_success() -> anyhow::Result<()> {
        let agent = Arc::new(MockAgent::new().with_asset("remote_file.txt", b"remote content"));
        let mut lib = StdAssetsLibrary::new();
        lib.add(Arc::new(AgentAssets::new(agent, vec!["remote_file.txt".to_string()])))?;
        let content = lib.read_binary("remote_file.txt".to_string());
        assert!(content.is_ok());
        assert_eq!(content.unwrap(), b"remote content");
        Ok(())
    }

    #[test]
    fn test_read_binary_remote_fail()-> anyhow::Result<()> {
        let agent = Arc::new(MockAgent::new().should_fail());
        let mut lib = StdAssetsLibrary::new();
        lib.add(Arc::new(AgentAssets::new(agent, vec!["remote_file.txt".to_string()])))?;
        let result = lib.read_binary("remote_file.txt".to_string());
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_read_embedded_success()-> anyhow::Result<()> {
        let agent = Arc::new(MockAgent::new());
        let mut lib = StdAssetsLibrary::new();
        lib.add(Arc::new(AgentAssets::new(agent, vec!["remote_file.txt".to_string()])))?;
        lib.add(Arc::new(EmbeddedAssets::<TestAsset>::new()))?;
        let content = lib.read("print/main.eldritch".to_string());
        assert!(content.is_ok());
        assert_eq!(content.unwrap(), "print(\"This script just prints\")\n");
        Ok(())
    }

    #[test]
    fn test_copy_success() -> anyhow::Result<()> {
        let agent = Arc::new(MockAgent::new());
        let mut lib = StdAssetsLibrary::new();
        lib.add(Arc::new(AgentAssets::new(agent, Vec::new())))?;
        lib.add(Arc::new(EmbeddedAssets::<TestAsset>::new()))?;
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_path = temp_dir.path().join("copied_main.eldritch");
        let dest_str = dest_path.to_str().unwrap().to_string();
        let result = lib.copy("print/main.eldritch".to_string(), dest_str.clone());
        assert!(result.is_ok());
        let content = std::fs::read_to_string(dest_path).unwrap();
        assert_eq!(content.trim(), "print(\"This script just prints\")");
    }

    #[test]
    fn test_copy_fail_read() -> anyhow::Result<()> {
        let agent = Arc::new(MockAgent::new());
        let mut lib = StdAssetsLibrary::new();
        lib.add(Arc::new(AgentAssets::new(agent, Vec::new())))?;
        lib.add(Arc::new(EmbeddedAssets::<TestAsset>::new()))?;
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_path = temp_dir.path().join("should_not_exist");
        let result = lib.copy(
            "nonexistent".to_string(),
            dest_path.to_str().unwrap().to_string(),
        );
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_copy_fail_write() -> anyhow::Result<()> {
        let agent = Arc::new(MockAgent::new());
        let mut lib = StdAssetsLibrary::new();
        lib.add(Arc::new(AgentAssets::new(agent, Vec::new())))?;
        lib.add(Arc::new(EmbeddedAssets::<TestAsset>::new()))?;
        let temp_dir = tempfile::tempdir().unwrap();
        let _dest_str = temp_dir.path().to_str().unwrap().to_string();
        let invalid_dest = temp_dir
            .path()
            .join("nonexistent_dir")
            .join("file.txt")
            .to_str()
            .unwrap()
            .to_string();
        let result = lib.copy("print/main.eldritch".to_string(), invalid_dest);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_list() -> anyhow::Result<()> {
        let agent = Arc::new(MockAgent::new());
        let remote_files = vec!["remote1.txt".to_string(), "remote2.txt".to_string()];
        let mut lib = StdAssetsLibrary::new();
        lib.add(Arc::new(AgentAssets::new(agent, remote_files.clone())))?;
        lib.add(Arc::new(EmbeddedAssets::<TestAsset>::new()))?;
        let list = lib.list().unwrap();
        assert!(list.iter().any(|f| f.contains("print/main.eldritch")));
        assert!(list.contains(&"remote1.txt".to_string()));
        assert!(list.contains(&"remote2.txt".to_string()));
        Ok(())
    }
}
