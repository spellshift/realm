use eldritch::Interpreter;
use eldritch::Value;
use eldritch::assets::std::{AgentAssets, StdAssetsLibrary};
use eldritch_agent::{Agent, Context};
use eldritch_mockagent::MockAgent;
use pb::c2;
use pb::c2::{ReportOutputRequest, TaskContext, TaskError, TaskOutput, report_output_request};
use std::sync::Arc;
use std::sync::Mutex;

struct FailingAgent {
    mock: MockAgent,
    reports: Mutex<Vec<ReportOutputRequest>>,
}

impl Agent for FailingAgent {
    fn report_output(&self, req: ReportOutputRequest) -> Result<c2::ReportOutputResponse, String> {
        self.reports.lock().unwrap().push(req);
        Ok(c2::ReportOutputResponse {})
    }

    fn fetch_asset(&self, req: c2::FetchAssetRequest) -> Result<Vec<u8>, String> {
        self.mock.fetch_asset(req)
    }

    // Proxy the rest
    fn report_credential(
        &self,
        req: c2::ReportCredentialRequest,
    ) -> Result<c2::ReportCredentialResponse, String> {
        self.mock.report_credential(req)
    }
    fn report_file(
        &self,
        req: std::sync::mpsc::Receiver<c2::ReportFileRequest>,
    ) -> Result<c2::ReportFileResponse, String> {
        self.mock.report_file(req)
    }
    fn report_process_list(
        &self,
        req: c2::ReportProcessListRequest,
    ) -> Result<c2::ReportProcessListResponse, String> {
        self.mock.report_process_list(req)
    }
    fn start_reverse_shell(&self, ctx: Context, cmd: Option<String>) -> Result<(), String> {
        self.mock.start_reverse_shell(ctx, cmd)
    }
    fn create_portal(&self, ctx: Context) -> Result<(), String> {
        self.mock.create_portal(ctx)
    }
    fn start_repl_reverse_shell(&self, ctx: Context) -> Result<(), String> {
        self.mock.start_repl_reverse_shell(ctx)
    }
    fn claim_tasks(&self, req: c2::ClaimTasksRequest) -> Result<c2::ClaimTasksResponse, String> {
        self.mock.claim_tasks(req)
    }
    fn get_config(&self) -> Result<std::collections::BTreeMap<String, String>, String> {
        self.mock.get_config()
    }
    fn get_transport(&self) -> Result<String, String> {
        self.mock.get_transport()
    }
    fn set_transport(&self, t: String) -> Result<(), String> {
        self.mock.set_transport(t)
    }
    fn list_transports(&self) -> Result<Vec<String>, String> {
        self.mock.list_transports()
    }
    fn get_callback_interval(&self) -> Result<u64, String> {
        self.mock.get_callback_interval()
    }
    fn set_callback_interval(&self, t: u64) -> Result<(), String> {
        self.mock.set_callback_interval(t)
    }
    fn set_callback_uri(&self, t: String) -> Result<(), String> {
        self.mock.set_callback_uri(t)
    }
    fn list_callback_uris(&self) -> Result<std::collections::BTreeSet<String>, String> {
        self.mock.list_callback_uris()
    }
    fn get_active_callback_uri(&self) -> Result<String, String> {
        self.mock.get_active_callback_uri()
    }
    fn get_next_callback_uri(&self) -> Result<String, String> {
        self.mock.get_next_callback_uri()
    }
    fn add_callback_uri(&self, t: String) -> Result<(), String> {
        self.mock.add_callback_uri(t)
    }
    fn remove_callback_uri(&self, t: String) -> Result<(), String> {
        self.mock.remove_callback_uri(t)
    }
    fn list_tasks(&self) -> Result<Vec<c2::Task>, String> {
        self.mock.list_tasks()
    }
    fn stop_task(&self, t: i64) -> Result<(), String> {
        self.mock.stop_task(t)
    }
}

#[test]
fn test_assets_copy_error_reported() {
    println!("Hello from proper test");
    let mut interp = Interpreter::new().with_default_libs();

    let mock = MockAgent::new().should_fail();
    let agent_impl = FailingAgent {
        mock,
        reports: Mutex::new(vec![]),
    };
    let agent: Arc<FailingAgent> = Arc::new(agent_impl);

    let mut assets_lib = StdAssetsLibrary::new();
    let context = Context::Task(TaskContext {
        task_id: 1,
        jwt: "".to_string(),
    });
    assets_lib
        .add(Arc::new(AgentAssets::new(
            agent.clone(),
            context.clone(),
            vec!["nonexistent".to_string()],
        )))
        .unwrap();

    interp.define_variable("assets", Value::Foreign(Arc::new(assets_lib)));

    let code = r#"
assets.copy("nonexistent", "dest")
"#;
    let result = interp.interpret(code);
    println!("Interpreter Result: {:?}", result);

    assert!(result.is_err());

    let (task_id, task_context) = match context {
        Context::Task(tc) => (tc.task_id, tc),
        _ => return,
    };

    match result {
        Ok(_) => panic!("Should not be ok"),
        Err(e) => {
            let _ = agent.report_output(ReportOutputRequest {
                message: Some(report_output_request::Message::TaskOutput(
                    pb::c2::ReportTaskOutputMessage {
                        context: Some(task_context.clone()),
                        output: Some(TaskOutput {
                            id: task_id,
                            output: String::new(),
                            error: Some(TaskError { msg: e }),
                            exec_started_at: None,
                            exec_finished_at: Some(prost_types::Timestamp::from(
                                std::time::SystemTime::now(),
                            )),
                        }),
                    },
                )),
            });
        }
    }

    let reports_lock = agent.reports.lock().unwrap();
    println!("Total reports: {}", reports_lock.len());
    let req = &reports_lock[0];
    let msg = req.message.as_ref().unwrap();
    if let report_output_request::Message::TaskOutput(out) = msg {
        let task_out = out.output.as_ref().unwrap();
        println!("Task Error: {:?}", task_out.error);
        assert!(task_out.error.is_some());
        let error_msg = task_out.error.as_ref().unwrap().msg.clone();
        assert!(error_msg.contains("Failed to fetch asset"));
    } else {
        panic!("Wrong message type");
    }
}
