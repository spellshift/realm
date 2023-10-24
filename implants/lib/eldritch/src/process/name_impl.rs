use anyhow::Result;
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};

pub fn name(pid: i32) -> Result<String> {
    if !System::IS_SUPPORTED {
        return Err(anyhow::anyhow!("This OS isn't supported for process functions.
         Pleases see sysinfo docs for a full list of supported systems.
         https://docs.rs/sysinfo/0.23.5/sysinfo/index.html#supported-oses\n\n"));
    }

    let mut sys = System::new();
    sys.refresh_processes();

    let mut res = "";

    if let Some(process) = sys.process(Pid::from_u32(pid as u32)) {
        res = process.name()
    }
    
    Ok(res.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{process::Command};

    #[test]
    fn test_process_name() -> anyhow::Result<()>{
        let mut commandstring = "";
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
        } else {
            return Err(anyhow::anyhow!("OS Not supported please re run on Linux, Windows, or MacOS"));
        }
        
        let child = Command::new(commandstring)
            .arg("10")
            .spawn()?;

        let mut sys = System::new();
        sys.refresh_processes();    
        for (pid, process) in sys.processes() {
            if pid.as_u32() == child.id() {
                let i32_pid = pid.as_u32() as i32;
                    let pname = name(i32_pid)?;
                    assert_eq!(process.name().to_string(), pname)
            }
        }
        return Ok(())
    }
}
