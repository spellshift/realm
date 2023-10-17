use starlark::{values::{dict::Dict, Heap, Value}, collections::SmallMap, const_frozen_string};
use anyhow::Result;
use std::process::id;

#[cfg(not(target_os = "linux"))]
pub fn info(starlark_heap: &Heap, pid: Option<i32>) -> Result<Dict> {
    return Err(anyhow!("Not implemented for this platform"));
}

#[cfg(target_os = "linux")]
pub fn info(starlark_heap: &Heap, pid: Option<i32>) -> Result<Dict> {
    let map: SmallMap<Value, Value> = SmallMap::new();
    // Create Dict type.
    let mut dict = Dict::new(map);
    
    let proc = procfs::process::Process::new(pid.unwrap_or(id().try_into()?))?;
    let stat = proc.stat()?;
    
    dict.insert_hashed(const_frozen_string!("pid").to_value().get_hashed()?, starlark_heap.alloc(stat.pid));
    dict.insert_hashed(const_frozen_string!("comm").to_value().get_hashed()?, starlark_heap.alloc(stat.comm));
    dict.insert_hashed(const_frozen_string!("state").to_value().get_hashed()?, starlark_heap.alloc(stat.state));
    dict.insert_hashed(const_frozen_string!("ppid").to_value().get_hashed()?, starlark_heap.alloc(stat.ppid));
    dict.insert_hashed(const_frozen_string!("pgrp").to_value().get_hashed()?, starlark_heap.alloc(stat.pgrp));
    dict.insert_hashed(const_frozen_string!("session").to_value().get_hashed()?, starlark_heap.alloc(stat.session));
    dict.insert_hashed(const_frozen_string!("tty_nr").to_value().get_hashed()?, starlark_heap.alloc(stat.tty_nr));
    dict.insert_hashed(const_frozen_string!("tpgid").to_value().get_hashed()?, starlark_heap.alloc(stat.tpgid));
    dict.insert_hashed(const_frozen_string!("flags").to_value().get_hashed()?, starlark_heap.alloc(stat.flags));
    dict.insert_hashed(const_frozen_string!("minflt").to_value().get_hashed()?, starlark_heap.alloc(stat.minflt));

    dict.insert_hashed(const_frozen_string!("cminflt").to_value().get_hashed()?, starlark_heap.alloc(stat.cminflt));
    dict.insert_hashed(const_frozen_string!("majflt").to_value().get_hashed()?, starlark_heap.alloc(stat.majflt));
    dict.insert_hashed(const_frozen_string!("cmajflt").to_value().get_hashed()?, starlark_heap.alloc(stat.cmajflt));
    dict.insert_hashed(const_frozen_string!("utime").to_value().get_hashed()?, starlark_heap.alloc(stat.utime));
    dict.insert_hashed(const_frozen_string!("stime").to_value().get_hashed()?, starlark_heap.alloc(stat.stime));
    dict.insert_hashed(const_frozen_string!("cutime").to_value().get_hashed()?, starlark_heap.alloc(stat.cutime));
    dict.insert_hashed(const_frozen_string!("cstime").to_value().get_hashed()?, starlark_heap.alloc(stat.cstime));
    dict.insert_hashed(const_frozen_string!("priority").to_value().get_hashed()?, starlark_heap.alloc(stat.priority));
    dict.insert_hashed(const_frozen_string!("nice").to_value().get_hashed()?, starlark_heap.alloc(stat.nice));
    dict.insert_hashed(const_frozen_string!("num_threads").to_value().get_hashed()?, starlark_heap.alloc(stat.num_threads));

    dict.insert_hashed(const_frozen_string!("itrealvalue").to_value().get_hashed()?, starlark_heap.alloc(stat.itrealvalue));
    dict.insert_hashed(const_frozen_string!("starttime").to_value().get_hashed()?, starlark_heap.alloc(stat.starttime));
    dict.insert_hashed(const_frozen_string!("vsize").to_value().get_hashed()?, starlark_heap.alloc(stat.vsize));
    dict.insert_hashed(const_frozen_string!("rss").to_value().get_hashed()?, starlark_heap.alloc(stat.rss));
    dict.insert_hashed(const_frozen_string!("rsslim").to_value().get_hashed()?, starlark_heap.alloc(stat.pid));
    dict.insert_hashed(const_frozen_string!("startcode").to_value().get_hashed()?, starlark_heap.alloc(stat.rsslim));
    dict.insert_hashed(const_frozen_string!("endcode").to_value().get_hashed()?, starlark_heap.alloc(stat.startcode));
    dict.insert_hashed(const_frozen_string!("startstack").to_value().get_hashed()?, starlark_heap.alloc(stat.endcode));
    dict.insert_hashed(const_frozen_string!("kstkesp").to_value().get_hashed()?, starlark_heap.alloc(stat.startstack));
    dict.insert_hashed(const_frozen_string!("kstkeip").to_value().get_hashed()?, starlark_heap.alloc(stat.kstkesp));

    dict.insert_hashed(const_frozen_string!("signal").to_value().get_hashed()?, starlark_heap.alloc(stat.kstkeip));
    dict.insert_hashed(const_frozen_string!("blocked").to_value().get_hashed()?, starlark_heap.alloc(stat.signal));
    dict.insert_hashed(const_frozen_string!("sigignore").to_value().get_hashed()?, starlark_heap.alloc(stat.blocked));
    dict.insert_hashed(const_frozen_string!("sigcatch").to_value().get_hashed()?, starlark_heap.alloc(stat.sigignore));
    dict.insert_hashed(const_frozen_string!("wchan").to_value().get_hashed()?, starlark_heap.alloc(stat.sigcatch));
    dict.insert_hashed(const_frozen_string!("nswap").to_value().get_hashed()?, starlark_heap.alloc(stat.wchan));
    dict.insert_hashed(const_frozen_string!("cnswap").to_value().get_hashed()?, starlark_heap.alloc(stat.nswap));
    dict.insert_hashed(const_frozen_string!("exit_signal").to_value().get_hashed()?, starlark_heap.alloc(stat.cnswap));
    dict.insert_hashed(const_frozen_string!("processor").to_value().get_hashed()?, starlark_heap.alloc(stat.exit_signal));
    dict.insert_hashed(const_frozen_string!("processor").to_value().get_hashed()?, starlark_heap.alloc(stat.processor));

    dict.insert_hashed(const_frozen_string!("rt_priority").to_value().get_hashed()?, starlark_heap.alloc(stat.rt_priority));
    dict.insert_hashed(const_frozen_string!("policy").to_value().get_hashed()?, starlark_heap.alloc(stat.policy));
    dict.insert_hashed(const_frozen_string!("delayacct_blkio_ticks").to_value().get_hashed()?, starlark_heap.alloc(stat.delayacct_blkio_ticks));
    dict.insert_hashed(const_frozen_string!("guest_Time").to_value().get_hashed()?, starlark_heap.alloc(stat.guest_time));
    dict.insert_hashed(const_frozen_string!("cguest_time").to_value().get_hashed()?, starlark_heap.alloc(stat.cguest_time));
    dict.insert_hashed(const_frozen_string!("start_data").to_value().get_hashed()?, starlark_heap.alloc(stat.start_data));
    dict.insert_hashed(const_frozen_string!("end_data").to_value().get_hashed()?, starlark_heap.alloc(stat.end_data));
    dict.insert_hashed(const_frozen_string!("start_brk").to_value().get_hashed()?, starlark_heap.alloc(stat.start_brk));
    dict.insert_hashed(const_frozen_string!("arg_start").to_value().get_hashed()?, starlark_heap.alloc(stat.arg_start));
    dict.insert_hashed(const_frozen_string!("arg_end").to_value().get_hashed()?, starlark_heap.alloc(stat.arg_end));

    dict.insert_hashed(const_frozen_string!("env_start").to_value().get_hashed()?, starlark_heap.alloc(stat.env_start));
    dict.insert_hashed(const_frozen_string!("env_end").to_value().get_hashed()?, starlark_heap.alloc(stat.env_end));
    dict.insert_hashed(const_frozen_string!("exit_code").to_value().get_hashed()?, starlark_heap.alloc(stat.exit_code));

    Ok(dict)
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;
    use starlark::values::{Heap, Value};
    use anyhow::{anyhow, Result};
    use std::process::id;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_info_linux() -> Result<()> {
        let test_heap = Heap::new();
        let res = info(&test_heap, None)?;
        assert!(res.get(const_frozen_string!("pid").to_value())?.ok_or(anyhow!("Could not find PID"))?.unpack_i32().ok_or(anyhow!("PID is not an i32"))? as u32 == id());
        Ok(())
    }
}

#[cfg(test)]
#[cfg(not(target_os = "linux"))]
mod tests {
    use super::*;
    use starlark::values::{Heap, Value};
    use anyhow::{anyhow, Result};

    #[test]
    fn test_info_not_linux() -> Result<()> {
        let test_heap = Heap::new();
        let res = info(&test_heap, None);
        assert!(res.is_err());
        Ok(())
    }
}