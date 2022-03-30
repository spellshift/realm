use anyhow::{Result};
use sysinfo::{ProcessExt,System,SystemExt,PidExt,User,UserExt};

pub struct ProcessRes {
    pid:        u32,
    ppid:       u32,
    status:     String,
    username:   String,
    path:       String,
    command:    String,
}

impl ToString for ProcessRes {
    #[inline]
    fn to_string(&self) -> String {
        return format!("{{pid:{},ppid:{},status:\"{}\",username:\"{}\",path:\"{}\",command:\"{}\"}}",
        &self.pid,
        &self.ppid,
        &self.status,
        &self.username,
        &self.path,
        &self.command,
    );
    }
}


// Returns PID, PPID, status, user, cmd, command
// REVIEW BLOCKER: Do we want to add environ? start_time? run_time? {cpu,memory,disk}_usage?
// https://docs.rs/sysinfo/0.23.5/sysinfo/struct.Process.html
pub fn list() -> Result<Vec<String>> {
    if !System::IS_SUPPORTED {
        return Err(anyhow::anyhow!("This OS isn't supported for process functions.
         Pleases see sysinfo docs for a full list of supported systems.
         https://docs.rs/sysinfo/0.23.5/sysinfo/index.html#supported-oses\n\n"));
    }

    let mut res : Vec<String> = Vec::new();
    let mut sys = System::new();
    sys.refresh_processes();
    sys.refresh_users_list();
    let user_list =  sys.users().clone();
    // for user in user_list {
    //     println!("{} is username {}", user.name(), *user.username());
    // }    

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
            username:   username_to_username(process.uid, user_list),
            path:       String::from(process.exe().to_str().unwrap()),
            command:    String::from(process.cmd().join(" ")),
        };
        res.push(tmprow.to_string());
    }
    Ok(res)
}

fn username_to_username(username: u32, user_list: &[User]) -> String {
    for user in user_list {
        if *user.uid() == username {
            return String::from(user.name());
        }
    }
    return String::from("");
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