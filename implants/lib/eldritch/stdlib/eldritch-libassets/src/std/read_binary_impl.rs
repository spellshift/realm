use crate::std::StdAssetsLibrary;
use alloc::vec::Vec;
use anyhow::Result;

impl StdAssetsLibrary {
    pub fn read_binary_impl(&self, name: &str) -> Result<Vec<u8>> {
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

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::std::{AgentAssets, AssetsLibrary, EmbeddedAssets};
    use alloc::collections::BTreeMap;
    use alloc::string::String;
    use alloc::string::ToString;
    use alloc::vec::Vec;
    use eldritch_agent::Agent;
    use pb::c2;
    use pb::c2::TaskContext;
    use std::collections::BTreeSet;
    use std::sync::{Arc, Mutex};

    #[cfg(debug_assertions)]
    #[derive(rust_embed::Embed)]
    #[folder = "../../../../../bin/embedded_files_test"]
    pub struct TestAsset;

    pub struct MockAgent {
        assets: Mutex<BTreeMap<String, Vec<u8>>>,
        should_fail_fetch: bool,
    }

    impl MockAgent {
        pub fn new() -> Self {
            Self {
                assets: Mutex::new(BTreeMap::new()),
                should_fail_fetch: false,
            }
        }

        pub fn with_asset(self, name: &str, content: &[u8]) -> Self {
            self.assets
                .lock()
                .unwrap()
                .insert(name.to_string(), content.to_vec());
            self
        }

        pub fn should_fail(mut self) -> Self {
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
        fn start_reverse_shell(
            &self,
            _task_context: TaskContext,
            _cmd: Option<String>,
        ) -> Result<(), String> {
            Ok(())
        }
        fn start_repl_reverse_shell(&self, _task_context: TaskContext) -> Result<(), String> {
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

        fn create_portal(&self, _task_context: TaskContext) -> std::result::Result<(), String> {
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
        lib.add(Arc::new(AgentAssets::new(
            agent,
            TaskContext {
                task_id: 0,
                jwt: String::new(),
            },
            vec!["remote_file.txt".to_string()],
        )))?;
        let content = lib.read_binary("remote_file.txt".to_string());
        assert!(content.is_ok());
        assert_eq!(content.unwrap(), b"remote content");
        Ok(())
    }

    #[test]
    fn test_read_binary_remote_fail() -> anyhow::Result<()> {
        let agent = Arc::new(MockAgent::new().should_fail());
        let mut lib = StdAssetsLibrary::new();
        lib.add(Arc::new(AgentAssets::new(
            agent,
            TaskContext {
                task_id: 0,
                jwt: String::new(),
            },
            vec!["remote_file.txt".to_string()],
        )))?;
        let result = lib.read_binary("remote_file.txt".to_string());
        assert!(result.is_err());
        Ok(())
    }
}
