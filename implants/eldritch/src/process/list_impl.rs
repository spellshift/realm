use anyhow::{Result};
use sysinfo::{ProcessExt,System,SystemExt,PidExt};

pub struct ProcessRes {
    pid:        u32,
    ppid:       u32,
    status:     String,
    uid:        u32,
    path:       String,
    command:    String,
}

impl ToString for ProcessRes {
    #[inline]
    fn to_string(&self) -> String {
        return format!("{{pid:{},ppid:{},status:\"{}\",uid:{},path:\"{}\",command:\"{}\"}}",
        &self.pid,
        &self.ppid,
        &self.status,
        &self.uid,
        &self.path,
        &self.command,
    );
    }
}


// Returns PID, PPID, status, user, cmd, command
// https://docs.rs/sysinfo/0.23.5/sysinfo/enum.ProcessStatus.html
pub fn list() -> Result<Vec<String>> {
    if !System::IS_SUPPORTED {
        return Err(anyhow::anyhow!("This OS isn't supported for process functions.
         Pleases see sysinfo docs for a full list of supported systems.
         https://docs.rs/sysinfo/0.23.5/sysinfo/index.html#supported-oses\n\n"));
    }

    let mut res : Vec<String> = Vec::new();
    let mut sys = System::new();
    sys.refresh_processes();
    println!("Here");
    for (pid, process) in sys.processes() {
        let mut tmp_ppid = 0;
        if  process.parent() != None {
            tmp_ppid = process.parent().unwrap().as_u32();
        }
        let tmprow = ProcessRes{ 
            pid:        pid.as_u32(),
            ppid:       tmp_ppid,
            status:     process.status().to_string(),
            uid:        process.uid,                                            //REVIEW BLOCKER: Do we want uid number or add some more sys functionality to pass back username.
            path:       String::from(process.exe().to_str().unwrap()),
            command:    String::from(process.cmd().join(" ")),
        };
        res.push(tmprow.to_string());
    }
    Ok(res)
}

// REVIEW BLOCKER: Not totally sure the right way to test this.
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_process_list() -> anyhow::Result<()>{
        let res = list()?;
        for proc in res{
            println!("{:?}", proc.to_string());
        }
        Ok(())
    }
}