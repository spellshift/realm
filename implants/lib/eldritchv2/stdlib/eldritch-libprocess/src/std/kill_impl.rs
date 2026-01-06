use alloc::format;
use alloc::string::ToString;
use sysinfo::{Pid, ProcessExt, Signal, System, SystemExt};

pub fn kill(pid: i64) -> Result<(), String> {
    if !System::IS_SUPPORTED {
        return Err("System not supported".to_string());
    }

    let mut sys = System::new();
    sys.refresh_processes();

    if let Some(process) = sys.process(Pid::from(pid as usize)) {
        if process.kill_with(Signal::Kill).unwrap_or(false) {
            Ok(())
        } else {
            Err(format!("Failed to kill process {pid}"))
        }
    } else {
        Err(format!("Process {pid} not found"))
    }
}

#[cfg(all(test, feature = "stdlib"))]
mod tests {
    use super::super::StdProcessLibrary;
    use super::super::ProcessLibrary;
    use ::std::process::Command;

    #[test]
    fn test_std_process_kill() {
        // Spawn a sleep process
        let mut cmd = Command::new("sleep");
        cmd.arg("10");

        // Handle windows
        #[cfg(windows)]
        let mut cmd = Command::new("ping");
        #[cfg(windows)]
        cmd.args(["-n", "10", "127.0.0.1"]);

        if let Ok(mut child) = cmd.spawn() {
            let pid = child.id() as i64;
            let lib = StdProcessLibrary;

            // Wait a bit for process to start?
            ::std::thread::sleep(::std::time::Duration::from_millis(100));

            // Check if it exists
            assert!(lib.name(pid).is_ok());

            // Kill it
            lib.kill(pid).unwrap();

            // Wait for it to die
            let _ = child.wait();
        } else {
            // If sleep command not found, skip?
        }
    }
}
