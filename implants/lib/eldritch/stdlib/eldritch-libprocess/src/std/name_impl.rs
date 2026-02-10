use alloc::format;
use alloc::string::{String, ToString};
use sysinfo::{IS_SUPPORTED_SYSTEM, Pid, System};

pub fn name(pid: i64) -> Result<String, String> {
    if !IS_SUPPORTED_SYSTEM {
        return Err("System not supported".to_string());
    }
    let sys = System::new_all();

    if let Some(process) = sys.process(Pid::from(pid as usize)) {
        Ok(process.name().to_string_lossy().into_owned())
    } else {
        Err(format!("Process {pid} not found"))
    }
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
