use super::super::insert_dict_kv;
use anyhow::Result;
use starlark::{
    collections::SmallMap,
    const_frozen_string,
    values::{dict::Dict, Heap, Value},
};
use std::process::id;
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};

pub fn info(starlark_heap: &'_ Heap, pid: Option<usize>) -> Result<Dict<'_>> {
    let map: SmallMap<Value, Value> = SmallMap::new();
    // Create Dict type.
    let mut dict = Dict::new(map);
    let pid = pid.unwrap_or(id() as usize);
    let s = System::new_all();
    if let Some(process) = s.process(Pid::from(pid)) {
        insert_dict_kv!(dict, starlark_heap, "pid", pid, i32);
        insert_dict_kv!(dict, starlark_heap, "name", process.name(), String);
        insert_dict_kv!(dict, starlark_heap, "cmd", process.cmd().join(" "), String);
        insert_dict_kv!(
            dict,
            starlark_heap,
            "exe",
            process.exe().display().to_string(),
            String
        );
        insert_dict_kv!(
            dict,
            starlark_heap,
            "environ",
            process.environ().join(","),
            String
        );
        insert_dict_kv!(
            dict,
            starlark_heap,
            "cwd",
            process.cwd().display().to_string(),
            String
        );
        insert_dict_kv!(
            dict,
            starlark_heap,
            "root",
            process.root().display().to_string(),
            String
        );
        insert_dict_kv!(dict, starlark_heap, "memory_usage", process.memory(), u64);
        insert_dict_kv!(
            dict,
            starlark_heap,
            "virtual_memory_usage",
            process.virtual_memory(),
            u64
        );
        match process.parent() {
            Some(pid) => {
                insert_dict_kv!(dict, starlark_heap, "ppid", pid.as_u32(), u32);
            }
            None => {
                insert_dict_kv!(dict, starlark_heap, "ppid", None);
            }
        }
        insert_dict_kv!(
            dict,
            starlark_heap,
            "status",
            process.status().to_string(),
            String
        );
        insert_dict_kv!(dict, starlark_heap, "start_time", process.start_time(), u64);
        insert_dict_kv!(dict, starlark_heap, "run_time", process.run_time(), u64);
        match process.group_id() {
            Some(gid) => {
                insert_dict_kv!(dict, starlark_heap, "gid", *gid, u32);
            }
            None => {
                insert_dict_kv!(dict, starlark_heap, "gid", None);
            }
        }

        match process.effective_group_id() {
            Some(egid) => {
                insert_dict_kv!(dict, starlark_heap, "egid", *egid, u32);
            }
            None => {
                insert_dict_kv!(dict, starlark_heap, "egid", None);
            }
        }

        #[cfg(not(windows))]
        {
            match process.session_id() {
                Some(sid) => {
                    insert_dict_kv!(dict, starlark_heap, "sid", sid.as_u32(), u32);
                }
                None => {
                    insert_dict_kv!(dict, starlark_heap, "sid", None);
                }
            }

            match process.user_id() {
                Some(uid) => {
                    insert_dict_kv!(dict, starlark_heap, "uid", **uid, u32);
                }
                None => {
                    insert_dict_kv!(dict, starlark_heap, "uid", None);
                }
            }

            match process.effective_user_id() {
                Some(euid) => {
                    insert_dict_kv!(dict, starlark_heap, "euid", **euid, u32);
                }
                None => {
                    insert_dict_kv!(dict, starlark_heap, "euid", None);
                }
            }
        }
        #[cfg(windows)]
        {
            match process.session_id() {
                Some(sid) => {
                    insert_dict_kv!(dict, starlark_heap, "sid", sid.to_string(), String);
                }
                None => {
                    insert_dict_kv!(dict, starlark_heap, "sid", None);
                }
            }

            match process.user_id() {
                Some(uid) => {
                    insert_dict_kv!(dict, starlark_heap, "uid", uid.to_string(), String);
                }
                None => {
                    insert_dict_kv!(dict, starlark_heap, "uid", None);
                }
            }

            match process.effective_user_id() {
                Some(euid) => {
                    insert_dict_kv!(dict, starlark_heap, "euid", euid.to_string(), String);
                }
                None => {
                    insert_dict_kv!(dict, starlark_heap, "euid", None);
                }
            }
        }
    }
    Ok(dict)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{anyhow, Result};
    use starlark::values::Heap;
    use std::process::id;

    #[test]
    fn test_info_default() -> Result<()> {
        let test_heap = Heap::new();
        let res = info(&test_heap, None)?;
        assert!(
            match res.get(const_frozen_string!("pid").to_value()) {
                Ok(v) => Ok(v),
                Err(err) => Err(err.into_anyhow()),
            }?
            .ok_or(anyhow!("Could not find PID"))?
            .unpack_i32()
            .ok_or(anyhow!("PID is not an i32"))? as u32
                == id()
        );
        Ok(())
    }
}
