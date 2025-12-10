use super::AssetsLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use anyhow::Result;
use eldritch_libagent::agent::Agent;
use eldritch_macros::eldritch_library_impl;
use pb::c2::FetchAssetRequest;
use std::io::Write;

#[eldritch_library_impl(AssetsLibrary)]
pub struct StdAssetsLibrary {
    pub agent: Arc<dyn Agent>,
    pub remote_assets: Vec<String>,
    pub embedded_assets: BTreeMap<String, Vec<u8>>,
}

impl core::fmt::Debug for StdAssetsLibrary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StdAssetsLibrary")
            .field("remote_assets", &self.remote_assets)
            .field("embedded_assets", &self.embedded_assets.keys())
            .finish()
    }
}

impl StdAssetsLibrary {
    pub fn new(
        agent: Arc<dyn Agent>,
        remote_assets: Vec<String>,
        embedded_assets: BTreeMap<String, Vec<u8>>,
    ) -> Self {
        Self {
            agent,
            remote_assets,
            embedded_assets,
        }
    }

    fn read_binary_embedded(&self, src: &str) -> Result<Vec<u8>> {
        if let Some(data) = self.embedded_assets.get(src) {
            Ok(data.clone())
        } else {
            Err(anyhow::anyhow!("Embedded file {src} not found."))
        }
    }

    fn _read_binary(&self, name: &str) -> Result<Vec<u8>> {
        if self.remote_assets.iter().any(|s| s == name) {
            let req = FetchAssetRequest {
                name: name.to_string(),
            };
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
        let mut files: Vec<String> = self.embedded_assets.keys().cloned().collect();
        // Append remote assets to the list if they are not already there
        for remote in &self.remote_assets {
            if !files.contains(remote) {
                files.push(remote.clone());
            }
        }
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use eldritch_libagent::fake::AgentFake;
    use pb::c2;
    use rust_embed::RustEmbed;
    use std::sync::Mutex;

    #[cfg(debug_assertions)]
    #[derive(RustEmbed)]
    #[folder = "../../../../../bin/embedded_files_test"]
    pub struct TestAsset;

    // Helper to load test assets into map
    fn get_test_assets() -> BTreeMap<String, Vec<u8>> {
        let mut assets = BTreeMap::new();
        for file in TestAsset::iter() {
            let name = file.as_ref().to_string();
            let content = TestAsset::get(&name).unwrap().data.to_vec();
            assets.insert(name, content);
        }
        assets
    }

    // Define a MockAgent that implements the Agent trait
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
            self.assets.lock().unwrap().insert(name.to_string(), content.to_vec());
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

        // Default implementations for other methods
        fn report_credential(&self, _req: c2::ReportCredentialRequest) -> Result<c2::ReportCredentialResponse, String> { Ok(c2::ReportCredentialResponse::default()) }
        fn report_file(&self, _req: c2::ReportFileRequest) -> Result<c2::ReportFileResponse, String> { Ok(c2::ReportFileResponse::default()) }
        fn report_process_list(&self, _req: c2::ReportProcessListRequest) -> Result<c2::ReportProcessListResponse, String> { Ok(c2::ReportProcessListResponse::default()) }
        fn report_task_output(&self, _req: c2::ReportTaskOutputRequest) -> Result<c2::ReportTaskOutputResponse, String> { Ok(c2::ReportTaskOutputResponse::default()) }
        fn reverse_shell(&self) -> Result<(), String> { Ok(()) }
        fn start_reverse_shell(&self, _task_id: i64, _cmd: Option<String>) -> Result<(), String> { Ok(()) }
        fn start_repl_reverse_shell(&self, _task_id: i64) -> Result<(), String> { Ok(()) }
        fn claim_tasks(&self, _req: c2::ClaimTasksRequest) -> Result<c2::ClaimTasksResponse, String> { Ok(c2::ClaimTasksResponse::default()) }
        fn get_config(&self) -> Result<BTreeMap<String, String>, String> { Ok(BTreeMap::new()) }
        fn get_transport(&self) -> Result<String, String> { Ok("mock".into()) }
        fn set_transport(&self, _transport: String) -> Result<(), String> { Ok(()) }
        fn add_transport(&self, _transport: String, _config: String) -> Result<(), String> { Ok(()) }
        fn list_transports(&self) -> Result<Vec<String>, String> { Ok(Vec::new()) }
        fn get_callback_interval(&self) -> Result<u64, String> { Ok(10) }
        fn set_callback_interval(&self, _interval: u64) -> Result<(), String> { Ok(()) }
        fn list_tasks(&self) -> Result<Vec<c2::Task>, String> { Ok(Vec::new()) }
        fn stop_task(&self, _task_id: i64) -> Result<(), String> { Ok(()) }
        fn get_config(&self) -> Result<BTreeMap<String, String>, String> { Ok(BTreeMap::new()) }
    }

    #[test]
    fn test_read_binary_embedded_success() {
        let agent = Arc::new(AgentFake::default());
        let lib = StdAssetsLibrary::new(agent, Vec::new(), get_test_assets());
        // Using an asset we know exists in bin/embedded_files_test
        let content = lib.read_binary("print/main.eldritch".to_string());
        assert!(content.is_ok());
        let content = content.unwrap();
        assert!(content.len() > 0);
        assert_eq!(std::str::from_utf8(&content).unwrap(), "print(\"This script just prints\")\n");
    }

    #[test]
    fn test_read_binary_embedded_fail() {
        let agent = Arc::new(AgentFake::default());
        let lib = StdAssetsLibrary::new(agent, Vec::new(), get_test_assets());
        assert!(lib.read_binary("nonexistent_file".to_string()).is_err());
    }

    #[test]
    fn test_read_binary_remote_success() {
        let agent = Arc::new(MockAgent::new().with_asset("remote_file.txt", b"remote content"));
        let lib = StdAssetsLibrary::new(agent, vec!["remote_file.txt".to_string()], BTreeMap::new());

        let content = lib.read_binary("remote_file.txt".to_string());
        assert!(content.is_ok());
        assert_eq!(content.unwrap(), b"remote content");
    }

    #[test]
    fn test_read_binary_remote_fail() {
        let agent = Arc::new(MockAgent::new().should_fail());
        let lib = StdAssetsLibrary::new(agent, vec!["remote_file.txt".to_string()], BTreeMap::new());

        let result = lib.read_binary("remote_file.txt".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_read_embedded_success() {
        let agent = Arc::new(AgentFake::default());
        let lib = StdAssetsLibrary::new(agent, Vec::new(), get_test_assets());
        let content = lib.read("print/main.eldritch".to_string());
        assert!(content.is_ok());
        assert_eq!(content.unwrap(), "print(\"This script just prints\")\n");
    }

    #[test]
    fn test_copy_success() {
        let agent = Arc::new(AgentFake::default());
        let lib = StdAssetsLibrary::new(agent, Vec::new(), get_test_assets());

        let temp_dir = tempfile::tempdir().unwrap();
        let dest_path = temp_dir.path().join("copied_main.eldritch");
        let dest_str = dest_path.to_str().unwrap().to_string();

        let result = lib.copy("print/main.eldritch".to_string(), dest_str.clone());
        assert!(result.is_ok());

        let content = std::fs::read_to_string(dest_path).unwrap();
        assert_eq!(content, "print(\"This script just prints\")\n");
    }

    #[test]
    fn test_copy_fail_read() {
        let agent = Arc::new(AgentFake::default());
        let lib = StdAssetsLibrary::new(agent, Vec::new(), BTreeMap::new());
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_path = temp_dir.path().join("should_not_exist");

        let result = lib.copy("nonexistent".to_string(), dest_path.to_str().unwrap().to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_fail_write() {
        let agent = Arc::new(AgentFake::default());
        let lib = StdAssetsLibrary::new(agent, Vec::new(), get_test_assets());

        // Trying to write to a directory path instead of a file should fail
        let temp_dir = tempfile::tempdir().unwrap();
        let _dest_str = temp_dir.path().to_str().unwrap().to_string();

        // On some OSes opening a directory for writing fails, or we can try a non-existent dir
        let invalid_dest = temp_dir.path().join("nonexistent_dir").join("file.txt").to_str().unwrap().to_string();

        let result = lib.copy("print/main.eldritch".to_string(), invalid_dest);
        assert!(result.is_err());
    }

    #[test]
    fn test_list() {
        let agent = Arc::new(MockAgent::new());
        let remote_files = vec!["remote1.txt".to_string(), "remote2.txt".to_string()];
        let lib = StdAssetsLibrary::new(agent, remote_files.clone(), get_test_assets());

        let list = lib.list().unwrap();

        // Check for embedded file
        assert!(list.iter().any(|f| f.contains("print/main.eldritch")));

        // Check for remote files
        assert!(list.contains(&"remote1.txt".to_string()));
        assert!(list.contains(&"remote2.txt".to_string()));
    }
}
