use starlark::{values::{dict::Dict, Heap, Value}, collections::SmallMap, const_frozen_string};
use anyhow::Result;
use sysinfo::{System, Pid, SystemExt, ProcessExt, PidExt};
use std::process::id;


pub fn info(starlark_heap: &Heap, pid: Option<usize>) -> Result<Dict> {
    let map: SmallMap<Value, Value> = SmallMap::new();
    // Create Dict type.
    let mut dict = Dict::new(map);
    let pid = pid.unwrap_or(id() as usize);
    let s = System::new_all();
    if let Some(process) = s.process(Pid::from(pid)) {
        dict.insert_hashed(const_frozen_string!("pid").to_value().get_hashed()?, starlark_heap.alloc(pid));
        dict.insert_hashed(const_frozen_string!("name").to_value().get_hashed()?, starlark_heap.alloc_str(process.name()).to_value());
        dict.insert_hashed(const_frozen_string!("cmd").to_value().get_hashed()?, starlark_heap.alloc(process.cmd()));
        dict.insert_hashed(const_frozen_string!("exe").to_value().get_hashed()?, starlark_heap.alloc_str(process.exe().display().to_string().as_str()).to_value());
        dict.insert_hashed(const_frozen_string!("environ").to_value().get_hashed()?, starlark_heap.alloc(process.environ()));
        dict.insert_hashed(const_frozen_string!("cwd").to_value().get_hashed()?, starlark_heap.alloc_str(process.cwd().display().to_string().as_str()).to_value());
        dict.insert_hashed(const_frozen_string!("root").to_value().get_hashed()?, starlark_heap.alloc_str(process.root().display().to_string().as_str()).to_value());
        dict.insert_hashed(const_frozen_string!("memory_usage").to_value().get_hashed()?, starlark_heap.alloc(process.memory()));
        dict.insert_hashed(const_frozen_string!("virtual_memory_usage").to_value().get_hashed()?, starlark_heap.alloc(process.virtual_memory()));
        dict.insert_hashed(const_frozen_string!("ppid").to_value().get_hashed()?, process.parent().map_or(Value::new_none(), |pid| starlark_heap.alloc(pid.as_u32())));
        dict.insert_hashed(const_frozen_string!("status").to_value().get_hashed()?, starlark_heap.alloc_str(process.status().to_string().as_str()).to_value());
        dict.insert_hashed(const_frozen_string!("start_time").to_value().get_hashed()?, starlark_heap.alloc(process.start_time()));
        dict.insert_hashed(const_frozen_string!("run_time").to_value().get_hashed()?, starlark_heap.alloc(process.run_time()));
        dict.insert_hashed(const_frozen_string!("gid").to_value().get_hashed()?, process.group_id().map_or(Value::new_none(), |gid| starlark_heap.alloc(*gid)));
        dict.insert_hashed(const_frozen_string!("egid").to_value().get_hashed()?, process.effective_group_id().map_or(Value::new_none(), |egid| starlark_heap.alloc(*egid)));
        #[cfg(not(windows))]
        {
            dict.insert_hashed(const_frozen_string!("sid").to_value().get_hashed()?, process.session_id().map_or(Value::new_none(), |sid| starlark_heap.alloc(sid.as_u32())));
            dict.insert_hashed(const_frozen_string!("uid").to_value().get_hashed()?, process.user_id().map_or(Value::new_none(), |uid| starlark_heap.alloc(**uid)));
            dict.insert_hashed(const_frozen_string!("euid").to_value().get_hashed()?, process.effective_user_id().map_or(Value::new_none(), |euid| starlark_heap.alloc(**euid)));
        }
        #[cfg(windows)]
        {
            dict.insert_hashed(const_frozen_string!("sid").to_value().get_hashed()?, process.session_id().map_or(Value::new_none(), |sid| starlark_heap.alloc_str(sid.to_string().as_str()).to_value()));
            dict.insert_hashed(const_frozen_string!("uid").to_value().get_hashed()?, process.user_id().map_or(Value::new_none(), |uid| starlark_heap.alloc_str(uid.to_string().as_str()).to_value()));
            dict.insert_hashed(const_frozen_string!("euid").to_value().get_hashed()?, process.effective_user_id().map_or(Value::new_none(), |euid| starlark_heap.alloc_str(euid.to_string().as_str()).to_value()));
        }
    }
    Ok(dict)
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlark::values::{Heap, Value};
    use anyhow::{anyhow, Result};
    use std::process::id;

    #[test]
    fn test_info_default() -> Result<()> {
        let test_heap = Heap::new();
        let res = info(&test_heap, None)?;
        assert!(res.get(const_frozen_string!("pid").to_value())?.ok_or(anyhow!("Could not find PID"))?.unpack_i32().ok_or(anyhow!("PID is not an i32"))? as u32 == id());
        Ok(())
    }
}
