use anyhow::Result;
use sysinfo::{ProcessExt,System,SystemExt,PidExt,Pid,Signal};

pub fn kill(pid: i32) -> Result<()> {
    if !System::IS_SUPPORTED {
        return Err(anyhow::anyhow!("This OS isn't supported for process functions.
         Pleases see sysinfo docs for a full list of supported systems.
         https://docs.rs/sysinfo/0.23.5/sysinfo/index.html#supported-oses\n\n"));
    }

    let mut sys = System::new();
    sys.refresh_processes();
    if let Some(process) = sys.process(Pid::from(pid)) {
        process.kill_with(Signal::Kill);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_process_kill() -> anyhow::Result<()>{
        let mut commandstring = "sleep 5";
        if cfg!(target_os = "linux") || 
                cfg!(target_os = "ios") || 
                cfg!(target_os = "macos") || 
                cfg!(target_os = "android") || 
                cfg!(target_os = "freebsd") || 
                cfg!(target_os = "openbsd") ||
                cfg!(target_os = "netbsd") {
            commandstring = "sleep";
        } else if cfg!(target_os = "windows") {
            commandstring = "timeout";
        }
        
        let child = Command::new(commandstring)
            .arg("8")
            .spawn()?;

        let mut sys = System::new();
        sys.refresh_processes();    
        for (pid, process) in sys.processes() {
            if pid.as_u32() == child.id(){
                let i32_pid = pid.as_u32() as i32;
                kill(i32_pid)?;
                println!("{:?}", process.status().to_string());
                assert_eq!(true, true)
            }
        } 
        sys.refresh_processes();    
        for (pid, process) in sys.processes() {
            if pid.as_u32() == child.id(){
                assert_eq!(process.status().to_string(), "Zombie")
            }
        }
        return Ok(())
    }
}
