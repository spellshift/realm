#[cfg(test)]
mod tests {
    use crate::Interpreter;
    use alloc::collections::{BTreeMap, BTreeSet};
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;
    use eldritch_agent::Context;
    use eldritch_libassets::std::EmptyAssets;
    use pb::c2;
    use pb::c2::TaskContext;
    use std::sync::Arc;
    use std::time::Duration;

    struct MockAgent;

    impl crate::Agent for MockAgent {
        fn report_process_list(
            &self,
            _req: c2::ReportProcessListRequest,
        ) -> Result<c2::ReportProcessListResponse, String> {
            Ok(c2::ReportProcessListResponse::default())
        }
        fn fetch_asset(&self, _req: c2::FetchAssetRequest) -> Result<Vec<u8>, String> {
            Ok(Vec::new())
        }
        fn report_credential(
            &self,
            _req: c2::ReportCredentialRequest,
        ) -> Result<c2::ReportCredentialResponse, String> {
            Ok(c2::ReportCredentialResponse::default())
        }
        fn report_file(
            &self,
            _req: std::sync::mpsc::Receiver<c2::ReportFileRequest>,
        ) -> Result<c2::ReportFileResponse, String> {
            Ok(c2::ReportFileResponse::default())
        }
        fn report_output(
            &self,
            _req: c2::ReportOutputRequest,
        ) -> Result<c2::ReportOutputResponse, String> {
            Ok(c2::ReportOutputResponse::default())
        }
        fn start_reverse_shell(
            &self,
            _context: Context,
            _cmd: Option<String>,
        ) -> Result<(), String> {
            Ok(())
        }
        fn create_portal(&self, _context: Context) -> Result<(), String> {
            Ok(())
        }
        fn start_repl_reverse_shell(&self, _context: Context) -> Result<(), String> {
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
            Ok("http".to_string())
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
        fn set_callback_uri(&self, _uri: String) -> Result<(), String> {
            Ok(())
        }
        fn list_callback_uris(&self) -> Result<BTreeSet<String>, String> {
            Ok(BTreeSet::new())
        }
        fn get_active_callback_uri(&self) -> Result<String, String> {
            Ok(String::new())
        }
        fn get_next_callback_uri(&self) -> Result<String, String> {
            Ok(String::new())
        }
        fn add_callback_uri(&self, _uri: String) -> Result<(), String> {
            Ok(())
        }
        fn remove_callback_uri(&self, _uri: String) -> Result<(), String> {
            Ok(())
        }
        fn list_tasks(&self) -> Result<Vec<c2::Task>, String> {
            Ok(Vec::new())
        }
        fn stop_task(&self, _task_id: i64) -> Result<(), String> {
            Ok(())
        }
    }

    fn run_script_with_timeout(script: &str) {
        let (tx, rx) = std::sync::mpsc::channel();
        let script = script.to_string();

        std::thread::spawn(move || {
            let agent = Arc::new(MockAgent);
            let task_context = TaskContext {
                task_id: 123,
                jwt: "test_jwt".to_string(),
            };
            let context = Context::Task(task_context);
            let backend = Arc::new(EmptyAssets {});

            let mut interp = Interpreter::new().with_default_libs().with_context(
                agent,
                context,
                Vec::new(),
                backend,
            );

            let res = interp.interpret(&script);
            let _ = tx.send(res);
        });

        match rx.recv_timeout(Duration::from_secs(10)) {
            Ok(Ok(_)) => {} // Script finished successfully
            Ok(Err(e)) => panic!("Script failed: {:?}", e),
            Err(_) => panic!("Script timed out after 10 seconds!"),
        }
    }

    #[test]
    fn test_process_list_tome_timeout() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = std::path::PathBuf::from(manifest_dir)
            .join("../../../../tavern/tomes/process_list/main.eldritch");
        let script = std::fs::read_to_string(path).unwrap();
        run_script_with_timeout(&script);
    }

    #[test]
    fn test_netstat_tome_timeout() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = std::path::PathBuf::from(manifest_dir)
            .join("../../../../tavern/tomes/netstat/main.eldritch");
        let script = std::fs::read_to_string(path).unwrap();
        run_script_with_timeout(&script);
    }

    #[test]
    fn test_get_net_info_tome_timeout() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = std::path::PathBuf::from(manifest_dir)
            .join("../../../../tavern/tomes/get_net_info/main.eldritch");
        let script = std::fs::read_to_string(path).unwrap();
        run_script_with_timeout(&script);
    }
}
