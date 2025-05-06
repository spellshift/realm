use crate::runtime::{
    messages::{AsyncMessage, ReportProcessListMessage},
    Environment,
};
use anyhow::Result;
use pb::eldritch::{process::Status, Process, ProcessList};
use starlark::collections::SmallMap;
use starlark::values::Value;

pub fn process_list(env: &Environment, process_list: Vec<SmallMap<String, Value>>) -> Result<()> {
    let mut pb_process_list = ProcessList { list: Vec::new() };
    for proc in process_list {
        pb_process_list.list.push(Process {
            pid: unpack_u64(&proc, "pid"),
            ppid: unpack_u64(&proc, "ppid"),
            name: unpack_string(&proc, "name"),
            principal: unpack_string(&proc, "username"),
            path: unpack_string(&proc, "path"),
            cmd: unpack_string(&proc, "command"),
            env: unpack_string(&proc, "env"),
            cwd: unpack_string(&proc, "cwd"),
            status: unpack_status(&proc).into(),
        })
    }

    env.send(AsyncMessage::from(ReportProcessListMessage {
        id: env.id(),
        list: pb_process_list,
    }))?;
    Ok(())
}

fn unpack_i32(proc: &SmallMap<String, Value>, key: &str) -> i32 {
    match proc.get(key) {
        Some(val) => val.unpack_i32().unwrap_or(0),
        None => 0,
    }
}
fn unpack_u64(proc: &SmallMap<String, Value>, key: &str) -> u64 {
    unpack_i32(proc, key) as u64
}

fn unpack_string(proc: &SmallMap<String, Value>, key: &str) -> String {
    match proc.get(key) {
        Some(v) => v.unpack_str().unwrap_or("").to_string(),
        None => String::from(""),
    }
}

fn unpack_status(proc: &SmallMap<String, Value>) -> Status {
    let val = unpack_string(proc, "status");
    let status_str = format!("STATUS_{}", val).to_uppercase();
    Status::from_str_name(status_str.as_str()).unwrap_or(Status::Unknown)
}

#[cfg(test)]
mod test {
    use crate::runtime::{messages::AsyncMessage, Message};
    use pb::eldritch::process::Status;
    use pb::eldritch::*;
    use std::collections::HashMap;

    macro_rules! process_list_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[tokio::test]
            async fn $name() {
                let tc: TestCase = $value;

                // Run Eldritch (until finished)
                let mut runtime = crate::start(tc.id, tc.tome).await;
                runtime.finish().await;

                // Read Messages
                let mut found = false;
                for msg in runtime.messages() {
                    if let Message::Async(AsyncMessage::ReportProcessList(m)) = msg {
                        assert_eq!(tc.want_proc_list, m.list);
                        found = true;
                    }
                }
                assert!(found);
            }
        )*
        }
    }

    struct TestCase {
        pub id: i64,
        pub tome: Tome,
        pub want_proc_list: ProcessList,
    }

    process_list_tests! {
            one_process: TestCase{
                id: 123,
                tome: Tome{
                    eldritch: String::from(r#"report.process_list([{"pid":5,"ppid":101,"name":"test","username":"root","path":"/bin/cat","env":"COOL=1","command":"cat","cwd":"/home/meow","status":"IDLE"}])"#),
                    parameters: HashMap::new(),
                    file_names: Vec::new(),
                },
                want_proc_list: ProcessList{list: vec![
                    Process{
                        pid: 5,
                        ppid: 101,
                        name: "test".to_string(),
                        principal: "root".to_string(),
                        path: "/bin/cat".to_string(),
                        env: "COOL=1".to_string(),
                        cmd: "cat".to_string(),
                        cwd: "/home/meow".to_string(),
                        status: Status::Idle.into(),
                    },
                ]},
            },
    }
}
