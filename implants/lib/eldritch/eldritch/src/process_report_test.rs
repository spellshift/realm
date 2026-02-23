#[cfg(feature = "stdlib")]
use crate::Agent;
#[cfg(feature = "stdlib")]
use crate::Interpreter;
#[cfg(feature = "stdlib")]
use alloc::collections::{BTreeMap, BTreeSet};
#[cfg(feature = "stdlib")]
use alloc::string::{String, ToString};
#[cfg(feature = "stdlib")]
use alloc::sync::Arc;
#[cfg(feature = "stdlib")]
use alloc::vec::Vec;
#[cfg(feature = "stdlib")]
use eldritch_agent::Context;
#[cfg(feature = "stdlib")]
use pb::c2;
#[cfg(feature = "stdlib")]
use pb::c2::TaskContext;

#[cfg(feature = "stdlib")]
struct MockAgent;

#[cfg(feature = "stdlib")]
impl Agent for MockAgent {
    fn report_process_list(
        &self,
        req: c2::ReportProcessListRequest,
    ) -> Result<c2::ReportProcessListResponse, String> {
        if let Some(list) = req.list {
            if list.list.is_empty() {
                return Err("Process list is empty".to_string());
            }
            for p in list.list {
                if p.principal.is_empty() {
                    return Err(alloc::format!(
                        "Principal is empty for process pid={}",
                        p.pid
                    ));
                }
            }
        } else {
            return Err("No process list received".to_string());
        }
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
        _req: alloc::boxed::Box<dyn Iterator<Item = c2::ReportFileRequest> + Send + 'static>,
    ) -> Result<c2::ReportFileResponse, String> {
        Ok(c2::ReportFileResponse::default())
    }
    fn report_output(
        &self,
        _req: c2::ReportOutputRequest,
    ) -> Result<c2::ReportOutputResponse, String> {
        Ok(c2::ReportOutputResponse::default())
    }
    fn start_reverse_shell(&self, _context: Context, _cmd: Option<String>) -> Result<(), String> {
        Ok(())
    }
    fn create_portal(&self, _context: Context) -> Result<(), String> {
        Ok(())
    }
    fn start_repl_reverse_shell(&self, _context: Context) -> Result<(), String> {
        Ok(())
    }
    fn claim_tasks(&self, _req: c2::ClaimTasksRequest) -> Result<c2::ClaimTasksResponse, String> {
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

#[test]
#[cfg(feature = "stdlib")]
fn test_report_process_list_integration() {
    {
        use eldritch_libassets::std::EmptyAssets;

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

        let code = "report.process_list(process.list())";
        let result = interp.interpret(code);

        assert!(result.is_ok(), "Interpretation failed: {:?}", result.err());
    }
}
