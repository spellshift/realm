
use alloc::format;
use alloc::string::{String, ToString};

#[cfg(not(target_os = "solaris"))]
use sysinfo::{Pid, ProcessExt, System, SystemExt};

#[cfg(target_os = "solaris")]
use crate::std::solaris_proc;

pub fn name(pid: i64) -> Result<String, String> {
    #[cfg(not(target_os = "solaris"))]
    {
        if !System::IS_SUPPORTED {
            return Err("System not supported".to_string());
        }
        let mut sys = System::new();
        sys.refresh_processes();

        if let Some(process) = sys.process(Pid::from(pid as usize)) {
            Ok(process.name().to_string())
        } else {
            Err(format!("Process {pid} not found"))
        }
    }

    #[cfg(target_os = "solaris")]
    {
        let info = solaris_proc::read_psinfo(pid as i32)
            .map_err(|e| format!("Process {} not found: {}", pid, e))?;

        let name = String::from_utf8_lossy(&info.pr_fname)
            .trim_matches(char::from(0))
            .to_string();
        Ok(name)
    }
}

#[cfg(all(test, feature = "stdlib"))]
mod tests {
    use super::super::ProcessLibrary;
    use super::super::StdProcessLibrary;

    #[test]
    fn test_std_process_errors() {
        let lib = StdProcessLibrary;
        let invalid_pid = 999999999;
        assert!(lib.info(Some(invalid_pid)).is_err());
        assert!(lib.name(invalid_pid).is_err());
        // kill test omitted as it might need permissions or signal handling specific checks
    }
}
