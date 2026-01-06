use alloc::collections::BTreeMap;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_agent::Agent;
use eldritch_core::Value;
use pb::{c2, eldritch};

pub fn process_list(
    agent: Arc<dyn Agent>,
    task_id: i64,
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
        // Ignoring status for now as mapping is not trivial without string-to-enum logic

        processes.push(eldritch::Process {
            pid,
            ppid,
            name,
            principal,
            path,
            cmd,
            env,
            cwd,
            status: 0, // UNSPECIFIED
        });
    }

    let req = c2::ReportProcessListRequest {
        task_id,
        list: Some(eldritch::ProcessList { list: processes }),
    };
    agent.report_process_list(req).map(|_| ())
}
