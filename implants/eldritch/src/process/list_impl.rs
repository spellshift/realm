use anyhow::{Result};
use sysinfo::{ProcessExt,System,SystemExt,PidExt};
use  std::fmt;
use sysinfo::{UserExt};

pub struct ProcessRes {
    pid:        u32,
    ppid:       u32,
    status:     String,
    username:   String,
    path:       String,
    command:    String,
    cwd:        String,
    environ:    String,
    name:       String,
}

impl fmt::Display for ProcessRes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{pid:{},ppid:{},status:\"{}\",username:\"{}\",path:\"{}\",\
        command:\"{}\",cwd:\"{}\",environ:\"{}\",name:\"{}\"}}",
        &self.pid,
        &self.ppid,
        &self.status,
        &self.username,
        &self.path,
        &self.command,
        &self.cwd,
        &self.environ,
        &self.name,
        )
    }
}

pub fn list() -> Result<Vec<String>> {
    if !System::IS_SUPPORTED {
        return Err(anyhow::anyhow!("This OS isn't supported for process functions.
         Pleases see sysinfo docs for a full list of supported systems.
         https://docs.rs/sysinfo/0.23.5/sysinfo/index.html#supported-oses\n\n"));
    }
    const UNKNOWN_USER: &str = "???";

    let mut res : Vec<String> = Vec::new();
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
    

        let tmprow = ProcessRes{
            pid:        pid.as_u32(),
            ppid:       tmp_ppid,
            status:     process.status().to_string(),
            username:   tmp_username,
            path:       String::from(process.exe().to_str().unwrap()),
            command:    String::from(process.cmd().join(" ")),
            cwd:        String::from(process.cwd().to_str().unwrap()),
            environ:    String::from(process.environ().join(" ")),
            name:       String::from(process.name())
        };
        res.push(tmprow.to_string());
    }
    Ok(res)
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

        let child = Command::new(sleep_str)
            .arg("5")
            .spawn()?;
    
        let res = list()?;
        let searchstring = String::from(format!("pid:{}", child.id()));
        for proc in res{
            if proc.as_str().contains(&searchstring) {
                assert_eq!(true, true);
                return Ok(())
            }
        }
        assert_eq!(true, false);
        return Ok(())
    }
}
