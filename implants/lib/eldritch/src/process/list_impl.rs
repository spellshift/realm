use anyhow::{Result};
use eldritch_types::proc::Proc;
use sysinfo::{ProcessExt,System,SystemExt,PidExt, Pid};
use sysinfo::{UserExt};

pub fn list() -> Result<Vec<Proc>> {
    if !System::IS_SUPPORTED {
        return Err(anyhow::anyhow!("This OS isn't supported for process functions.
         Pleases see sysinfo docs for a full list of supported systems.
         https://docs.rs/sysinfo/0.23.5/sysinfo/index.html#supported-oses\n\n"));
    }
    const UNKNOWN_USER: &str = "???";

    let mut final_res: Vec<Proc> = Vec::new();
    let mut sys = System::new();
    sys.refresh_processes();
    sys.refresh_users_list();

    for (tmp_pid, process) in sys.processes() {
        let ppid = process.parent().unwrap_or(Pid::from(0)).as_u32();
        let pid = tmp_pid.as_u32();
        let username = match process.user_id() {
            Some(local_user_id) => match sys.get_user_by_id(local_user_id){
                    Some(local_username) => local_username.name().to_string(),
                    None => String::from(UNKNOWN_USER),
            },
            None => String::from(UNKNOWN_USER),
        };
        let status = process.status().to_string();
        let name = process.name().to_string();
        let path = String::from(process.exe().to_string_lossy());
        let command = String::from(process.cmd().join(" "));
        let cwd = String::from(process.cwd().to_string_lossy());
        let environ = String::from(process.environ().join(" "));

        final_res.push(Proc {
            pid, 
            ppid,
            status, 
            name, 
            path, 
            username, 
            command, 
            cwd,
            environ,
        });
    }
    Ok(final_res)
}


#[cfg(test)]
mod tests {
    use anyhow::Context;

    use super::*;
    use std::process::Command;

    #[test]
    fn test_process_list() -> anyhow::Result<()>{
        #[cfg(not(target_os = "windows"))]
        let sleep_str = "sleep";
        #[cfg(target_os = "windows")]
        let sleep_str = "timeout";

        let child = Command::new(sleep_str)
            .arg("5")
            .spawn()?;
    
        let res = list()?;
        for proc in res{
            if proc.pid == child.id() {
                assert!(true);
                return Ok(())
            }
        }
        assert!(false);
        return Ok(())
    }
}
