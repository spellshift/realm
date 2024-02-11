use anyhow::Result;
use starlark::values::Value;
use starlark::{collections::SmallMap, eval::Evaluator};

use crate::{
    pb::{process::Status, Process, ProcessList},
    runtime::Client,
};

pub fn process_list(
    starlark_eval: &Evaluator<'_, '_>,
    process_list: Vec<SmallMap<String, Value>>,
) -> Result<()> {
    let client = Client::from_extra(starlark_eval.extra)?;

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

    client.report_process_list(pb_process_list)?;
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
