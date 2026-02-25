use alloc::format;
use alloc::string::{String, ToString};

#[cfg(not(target_os = "solaris"))]
use sysinfo::{Pid, ProcessExt, System, SystemExt};

#[cfg(not(target_os = "solaris"))]
pub fn name(pid: i64) -> Result<String, String> {
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
pub fn name(pid: i64) -> Result<String, String> {
    use std::process::Command;

    // ps -p <pid> -o comm
    let output = Command::new("ps")
        .args(&["-p", &pid.to_string(), "-o", "comm"])
        .output()
        .map_err(|e| format!("Failed to execute ps: {}", e))?;

    if !output.status.success() {
        return Err(format!("Process {} not found", pid));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    if lines.len() < 2 {
        return Err(format!("Process {} not found", pid));
    }

    // Header is first line
    Ok(lines[1].trim().to_string())
}

#[cfg(all(test, feature = "stdlib"))]
mod tests {
    use super::super::ProcessLibrary;
    use super::super::StdProcessLibrary;

    #[test]
    fn test_std_process_errors() {
        let lib = StdProcessLibrary;
        // Using a very large PID that shouldn't exist
        let invalid_pid = 999999999;

        assert!(lib.info(Some(invalid_pid)).is_err());
        assert!(lib.name(invalid_pid).is_err());
        assert!(lib.kill(invalid_pid).is_err());
    }
}
