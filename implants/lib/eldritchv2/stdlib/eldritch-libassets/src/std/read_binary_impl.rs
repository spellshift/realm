use crate::RustEmbed;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use anyhow::Result;
use eldritch_agent::Agent;
use pb::c2::FetchAssetRequest;

pub fn read_binary_embedded<A: RustEmbed>(src: &str) -> Result<Vec<u8>> {
    if let Some(file) = A::get(src) {
        Ok(file.data.to_vec())
    } else {
        Err(anyhow::anyhow!("Embedded file {src} not found."))
    }
}

pub fn read_binary<A: RustEmbed>(
    agent: Arc<dyn Agent>,
    jwt: String,
    remote_assets: &[String],
    name: String,
) -> Result<Vec<u8>, String> {
    if remote_assets.iter().any(|s| s == &name) {
        let req = FetchAssetRequest { name, jwt };
        return agent.fetch_asset(req);
    }
    read_binary_embedded::<A>(&name).map_err(|e| e.to_string())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use alloc::collections::BTreeMap;
    use alloc::string::ToString;
    use pb::c2;
    use std::borrow::Cow;
    use std::collections::BTreeSet;
    use std::sync::Mutex;

    use crate::RustEmbed as LocalRustEmbed;
    use rust_embed::RustEmbed as CrateRustEmbed;

    #[cfg(debug_assertions)]
    #[derive(CrateRustEmbed)]
    #[folder = "../../../../../bin/embedded_files_test"]
    pub struct TestAsset;

    impl LocalRustEmbed for TestAsset {
        fn get(file_path: &str) -> Option<rust_embed::EmbeddedFile> {
            <TestAsset as CrateRustEmbed>::get(file_path)
        }
        fn iter() -> impl Iterator<Item = Cow<'static, str>> {
            <TestAsset as CrateRustEmbed>::iter()
        }
    }

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
            _task_id: i64,
            _jwt: String,
            _cmd: Option<String>,
        ) -> Result<(), String> {
            Ok(())
        }
        fn start_repl_reverse_shell(&self, _task_id: i64, _jwt: String) -> Result<(), String> {
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

        fn create_portal(&self, task_id: i64, _jwt: String) -> std::result::Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn test_read_binary_embedded_success() {
        let agent = Arc::new(MockAgent::new());
        let content = read_binary::<TestAsset>(
            agent,
            "a jwt".to_string(),
            &Vec::new(),
            "print/main.eldritch".to_string(),
        );
        assert!(content.is_ok());
        let content = content.unwrap();
        assert!(!content.is_empty());
        assert_eq!(
            std::str::from_utf8(&content).unwrap().trim(),
            "print(\"This script just prints\")"
        );
    }

    #[test]
    fn test_read_binary_embedded_fail() {
        let agent = Arc::new(MockAgent::new());
        assert!(
            read_binary::<TestAsset>(
                agent,
                "a jwt".to_string(),
                &Vec::new(),
                "nonexistent_file".to_string()
            )
            .is_err()
        );
    }

    #[test]
    fn test_read_binary_remote_success() {
        let agent = Arc::new(MockAgent::new().with_asset("remote_file.txt", b"remote content"));
        let content = read_binary::<TestAsset>(
            agent,
            "a jwt".to_string(),
            &vec!["remote_file.txt".to_string()],
            "remote_file.txt".to_string(),
        );
        assert!(content.is_ok());
        assert_eq!(content.unwrap(), b"remote content");
    }

    #[test]
    fn test_read_binary_remote_fail() {
        let agent = Arc::new(MockAgent::new().should_fail());
        let result = read_binary::<TestAsset>(
            agent,
            "a jwt".to_string(),
            &vec!["remote_file.txt".to_string()],
            "remote_file.txt".to_string(),
        );
        assert!(result.is_err());
    }
}
