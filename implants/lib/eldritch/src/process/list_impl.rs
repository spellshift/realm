use super::super::insert_dict_kv;
use anyhow::{Context, Result};
use starlark::{
    collections::SmallMap,
    const_frozen_string,
    values::{dict::Dict, Heap, Value},
};
use sysinfo::{PidExt, ProcessExt, System, SystemExt, UserExt};

pub fn list(starlark_heap: &Heap) -> Result<Vec<Dict<'_>>> {
    if !System::IS_SUPPORTED {
        return Err(anyhow::anyhow!(
            "This OS isn't supported for process functions.
         Pleases see sysinfo docs for a full list of supported systems.
         https://docs.rs/sysinfo/0.23.5/sysinfo/index.html#supported-oses\n\n"
        ));
    }
    const UNKNOWN_USER: &str = "???";

    let mut final_res: Vec<Dict> = Vec::new();
    let mut sys = System::new();
    sys.refresh_processes();
    sys.refresh_users_list();

    for (pid, process) in sys.processes() {
        let mut tmp_ppid = 0;
        if process.parent().is_some() {
            tmp_ppid = process
                .parent()
                .context(format!("Failed to get parent process for {}", pid))?
                .as_u32();
        }
        let tmp_username = match process.user_id() {
            Some(local_user_id) => match sys.get_user_by_id(local_user_id) {
                Some(local_username) => local_username.name().to_string(),
                None => String::from(UNKNOWN_USER),
            },
            None => String::from(UNKNOWN_USER),
        };

        let res: SmallMap<Value, Value> = SmallMap::new();
        // Create Dict type.
        let mut tmp_res = Dict::new(res);

        insert_dict_kv!(tmp_res, starlark_heap, "pid", pid.as_u32(), u32);
        insert_dict_kv!(tmp_res, starlark_heap, "ppid", tmp_ppid, u32);
        insert_dict_kv!(
            tmp_res,
            starlark_heap,
            "status",
            &process.status().to_string(),
            String
        );
        insert_dict_kv!(tmp_res, starlark_heap, "username", &tmp_username, String);
        insert_dict_kv!(
            tmp_res,
            starlark_heap,
            "path",
            process
                .exe()
                .to_str()
                .context("Failed to cast process exe to str")?,
            String
        );
        insert_dict_kv!(
            tmp_res,
            starlark_heap,
            "command",
            process.cmd().join(" "),
            String
        );
        insert_dict_kv!(
            tmp_res,
            starlark_heap,
            "cwd",
            process
                .cwd()
                .to_str()
                .context("Failed to cast cwd to str")?,
            String
        );
        insert_dict_kv!(
            tmp_res,
            starlark_heap,
            "environ",
            process.environ().join(" "),
            String
        );
        insert_dict_kv!(tmp_res, starlark_heap, "name", process.name(), String);

        final_res.push(tmp_res);
    }
    Ok(final_res)
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use super::*;
    use std::process::Command;

    #[test]
    fn test_process_list() -> anyhow::Result<()> {
        #[cfg(not(target_os = "windows"))]
        let sleep_str = "sleep";
        #[cfg(target_os = "windows")]
        let sleep_str = "timeout";

        let child = Command::new(sleep_str).arg("5").spawn()?;

        let binding = Heap::new();
        let res = list(&binding)?;
        for proc in res {
            println!(
                "{:?}",
                match proc.get(const_frozen_string!("pid").to_value()) {
                    Ok(v) => Ok(v),
                    Err(err) => Err(err.into_anyhow().context("fail")),
                }?
            );
            let cur_pid = match proc.get(const_frozen_string!("pid").to_value()) {
                Ok(v) => match v {
                    Some(local_cur_pid) => local_cur_pid
                        .unpack_i32()
                        .context("Failed to unpack starlark int to i32")?,
                    None => return Err(anyhow::anyhow!("pid couldn't be unwrapped")),
                },
                Err(err) => return Err(err.into_anyhow()),
            };
            if cur_pid as u32 == child.id() {
                assert_eq!(true, true);
                return Ok(());
            }
        }
        println!("PID: {}", child.id());
        assert_eq!(true, false);
        Ok(())
    }
}
