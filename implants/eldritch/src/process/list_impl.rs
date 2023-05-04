use anyhow::{Result};
use starlark::{values::{dict::Dict, Heap, Value}, collections::SmallMap, const_frozen_string};
use sysinfo::{ProcessExt,System,SystemExt,PidExt};
use std::fmt;
use sysinfo::{UserExt};

pub fn list(starlark_heap: &Heap) -> Result<Vec<Dict>> {
    if !System::IS_SUPPORTED {
        return Err(anyhow::anyhow!("This OS isn't supported for process functions.
         Pleases see sysinfo docs for a full list of supported systems.
         https://docs.rs/sysinfo/0.23.5/sysinfo/index.html#supported-oses\n\n"));
    }
    const UNKNOWN_USER: &str = "???";

    let mut final_res: Vec<Dict> = Vec::new();
    let mut sys = System::new();
    sys.refresh_processes();
    sys.refresh_users_list();

    for (pid, process) in sys.processes() {
        let mut tmp_ppid = 0;
        if  process.parent() != None {
            tmp_ppid = process.parent().unwrap().as_u32();
        }
        let tmp_username = match process.user_id() {
            Some(local_user_id) => match sys.get_user_by_id(local_user_id){
                    Some(local_username) => local_username.name().to_string(),
                    None => String::from(UNKNOWN_USER),
            },
            None => String::from(UNKNOWN_USER),
        };
    

        let res: SmallMap<Value, Value> = SmallMap::new();
        // Create Dict type.
        let mut tmp_res = Dict::new(res);

        tmp_res.insert_hashed(const_frozen_string!("pid").to_value().get_hashed().unwrap(), Value::new_int(match pid.as_u32().try_into() {
            Ok(local_int) => local_int,
            Err(_) => -1,
        }));
        tmp_res.insert_hashed(const_frozen_string!("ppid").to_value().get_hashed().unwrap(), Value::new_int(match tmp_ppid.try_into() {
            Ok(local_int) => local_int,
            Err(_) => -1,
        }));
        tmp_res.insert_hashed(const_frozen_string!("status").to_value().get_hashed().unwrap(), starlark_heap.alloc_str(&process.status().to_string()).to_value());
        tmp_res.insert_hashed(const_frozen_string!("username").to_value().get_hashed().unwrap(), starlark_heap.alloc_str(&tmp_username).to_value());
        tmp_res.insert_hashed(const_frozen_string!("path").to_value().get_hashed().unwrap(), starlark_heap.alloc_str(&String::from(process.exe().to_str().unwrap())).to_value());
        tmp_res.insert_hashed(const_frozen_string!("command").to_value().get_hashed().unwrap(), starlark_heap.alloc_str(&String::from(process.cmd().join(" "))).to_value());
        tmp_res.insert_hashed(const_frozen_string!("cwd").to_value().get_hashed().unwrap(), starlark_heap.alloc_str(&String::from(process.cwd().to_str().unwrap())).to_value());
        tmp_res.insert_hashed(const_frozen_string!("environ").to_value().get_hashed().unwrap(), starlark_heap.alloc_str(&String::from(process.environ().join(" "))).to_value());
        tmp_res.insert_hashed(const_frozen_string!("name").to_value().get_hashed().unwrap(), starlark_heap.alloc_str(&String::from(process.name())).to_value());

        final_res.push(tmp_res);
    }
    Ok(final_res)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_process_list() -> anyhow::Result<()>{
        #[cfg(not(target_os = "windows"))]
        let sleep_str = "sleep";
        #[cfg(target_os = "windows")]
        let sleep_str = "timeout";

        let mut child = Command::new(sleep_str)
            .arg("5")
            .spawn()?;
    
        let binding = Heap::new();
        let res = list(&binding)?;
        for proc in res{
            let cur_pid = match proc.get(const_frozen_string!("pid").to_value())? {
                Some(local_cur_pid) => local_cur_pid.to_int()?,
                None => return Err(anyhow::anyhow!("pid couldn't be unwrapped")),
            };
            if cur_pid as u32 == child.id() {
                assert_eq!(true, true);
                return Ok(())
            }
        }
        assert_eq!(true, false);
        return Ok(())
    }
}
