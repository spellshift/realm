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

        let status = match d.get("status").map(|v| v.to_string()).as_deref() {
            Some("Idle") => eldritch::process::Status::Idle as i32,
            Some("Run") => eldritch::process::Status::Run as i32,
            Some("Sleep") => eldritch::process::Status::Sleep as i32,
            Some("Stop") => eldritch::process::Status::Stop as i32,
            Some("Zombie") => eldritch::process::Status::Zombie as i32,
            Some("Tracing") => eldritch::process::Status::Tracing as i32,
            Some("Dead") => eldritch::process::Status::Dead as i32,
            Some("WakeKill") => eldritch::process::Status::WakeKill as i32,
            Some("Waking") => eldritch::process::Status::Waking as i32,
            Some("Parked") => eldritch::process::Status::Parked as i32,
            Some("LockBlocked") => eldritch::process::Status::LockBlocked as i32,
            Some("UninteruptibleDiskSleep") => eldritch::process::Status::UninteruptibleDiskSleep as i32,
            Some("Unknown") => eldritch::process::Status::Unknown as i32,
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
