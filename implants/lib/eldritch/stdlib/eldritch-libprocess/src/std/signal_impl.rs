use alloc::format;
#[cfg(windows)]
use alloc::string::ToString;

#[cfg(unix)]
pub fn signal(pid: i64, signal: i64) -> Result<(), String> {
    let ret = unsafe { libc::kill(pid as libc::pid_t, signal as libc::c_int) };
    if ret == 0 {
        Ok(())
    } else {
        let err = ::std::io::Error::last_os_error();
        Err(format!(
            "Failed to send signal {signal} to process {pid}: {err}"
        ))
    }
}

#[cfg(windows)]
pub fn signal(pid: i64, signal: i64) -> Result<(), String> {
    use sysinfo::{Pid, ProcessExt, Signal, System, SystemExt};

    let sig = match signal {
        9 => Signal::Kill,
        _ => {
            return Err(format!(
                "Signal {signal} is not supported on Windows. Only signal 9 (Kill) is supported."
            ));
        }
    };

    if !System::IS_SUPPORTED {
        return Err("System not supported".to_string());
    }

    let mut sys = System::new();
    sys.refresh_processes();

    if let Some(process) = sys.process(Pid::from(pid as usize)) {
        if process.kill_with(sig).unwrap_or(false) {
            Ok(())
        } else {
            Err(format!("Failed to send signal {signal} to process {pid}"))
        }
    } else {
        Err(format!("Process {pid} not found"))
    }
}

#[cfg(all(test, feature = "stdlib"))]
mod tests {
    use super::super::ProcessLibrary;
    use super::super::StdProcessLibrary;
    use ::std::process::Command;

    #[test]
    fn test_std_process_signal() {
        let mut cmd = Command::new("sleep");
        cmd.arg("10");

        #[cfg(windows)]
        let mut cmd = Command::new("ping");
        #[cfg(windows)]
        cmd.args(["-n", "10", "127.0.0.1"]);

        if let Ok(mut child) = cmd.spawn() {
            let pid = child.id() as i64;
            let lib = StdProcessLibrary;

            ::std::thread::sleep(::std::time::Duration::from_millis(100));

            // Verify the process exists
            assert!(lib.name(pid).is_ok());

            // Send SIGKILL (9) to stop the process
            lib.signal(pid, 9).unwrap();

            // Wait for the process to terminate
            let _ = child.wait();
        }
    }
}
