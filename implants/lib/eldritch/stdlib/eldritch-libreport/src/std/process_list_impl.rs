use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_agent::{Agent, Context};
use eldritch_core::Value;
use pb::c2::report_process_list_request;
use pb::{c2, eldritch};

pub fn map_status(status_str: &str) -> i32 {
    match status_str {
        "Idle" | "Idle " => eldritch::process::Status::Idle as i32,
        "Run" | "Running" => eldritch::process::Status::Run as i32,
        "Sleep" | "Sleeping" => eldritch::process::Status::Sleep as i32,
        "Stop" | "Stopped" => eldritch::process::Status::Stop as i32,
        "Zombie" | "Defunct" => eldritch::process::Status::Zombie as i32,
        "Tracing" | "TracingStop" => eldritch::process::Status::Tracing as i32,
        "Dead" | "Dead " => eldritch::process::Status::Dead as i32,
        "WakeKill" | "Wakekill" => eldritch::process::Status::WakeKill as i32,
        "Waking" => eldritch::process::Status::Waking as i32,
        "Parked" | "Parked " => eldritch::process::Status::Parked as i32,
        "LockBlocked" => eldritch::process::Status::LockBlocked as i32,
        "UninterruptibleDiskSleep" | "UninteruptibleDiskSleep" => {
            eldritch::process::Status::UninteruptibleDiskSleep as i32
        }
        "Unknown" => eldritch::process::Status::Unknown as i32,
        _ => eldritch::process::Status::Unspecified as i32,
    }
}

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

        let status_str = d
            .get("status")
            .and_then(|v| match v {
                Value::String(s) => Some(s.as_str()),
                _ => None,
            })
            .unwrap_or("Unknown");

        let status = map_status(status_str);

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
    use eldritch_core::Value;

    #[test]
    fn test_process_list_status_mapping() {
        use ::std::process::Command;
        use eldritch_libprocess::ProcessLibrary;
        use eldritch_libprocess::std::StdProcessLibrary;

        // Spawn a process to ensure we have at least one active process we can inspect
        let mut cmd = Command::new("sleep");
        cmd.arg("10");

        #[cfg(windows)]
        let mut cmd = Command::new("ping");
        #[cfg(windows)]
        cmd.args(["-n", "10", "127.0.0.1"]);

        if let Ok(mut child) = cmd.spawn() {
            let pid = child.id() as i64;

            ::std::thread::sleep(::std::time::Duration::from_millis(100));

            let lib = StdProcessLibrary;
            let list = lib.list().unwrap();
            assert!(!list.is_empty());

            // Find our spawned process
            let my_proc = list
                .iter()
                .find(|p| {
                    if let Some(Value::Int(p_pid)) = p.get("pid") {
                        *p_pid == pid
                    } else {
                        false
                    }
                })
                .expect("Could not find spawned process");

            let status_str = my_proc
                .get("status")
                .and_then(|v| match v {
                    Value::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .unwrap_or("Unknown");

            let status = map_status(status_str);

            println!(
                "Test debug info - PID: {}, Status Str: {}, Mapped Status: {}",
                pid, status_str, status
            );

            // The spawned process should have a valid (non-unspecified) status, likely "Run" or "Sleep"
            assert_ne!(
                status,
                eldritch::process::Status::Unspecified as i32,
                "Process status should not be unspecified for actively running process"
            );

            // Cleanup
            let _ = child.kill();
            let _ = child.wait();
        } else {
            panic!("Could not spawn test process");
        }
    }
}
