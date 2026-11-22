use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::Guardrail;

#[derive(Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Process {
    pub name: String,
}

impl Process {
    pub fn new(name: &str) -> Self {
        Process {
            name: name.to_string(),
        }
    }
}

/// Match a target process name case-insensitively against:
/// 1. `process.name()` (may be truncated to 15 bytes on Linux via `/proc/<pid>/comm`),
/// 2. `process.exe()` basename (untruncated executable filename when readable),
/// 3. `process.cmd()[0]` basename (untruncated argv[0] fallback).
fn matches_process_name(target_lower: &str, name: &str, exe: &Path, cmd: &[String]) -> bool {
    if name.to_lowercase() == target_lower {
        return true;
    }

    if let Some(exe_name) = exe.file_name().and_then(|n| n.to_str()) {
        if exe_name.to_lowercase() == target_lower {
            return true;
        }
    }

    if let Some(arg0_name) = cmd
        .first()
        .and_then(|arg0| Path::new(arg0).file_name())
        .and_then(|n| n.to_str())
    {
        if arg0_name.to_lowercase() == target_lower {
            return true;
        }
    }

    false
}

impl Guardrail for Process {
    fn get_name(&self) -> String {
        "process".to_string()
    }

    fn check(&self) -> bool {
        use sysinfo::{ProcessExt, System, SystemExt};

        if self.name.is_empty() {
            return false;
        }

        let mut sys = System::new();
        sys.refresh_processes();

        let check_name = self.name.to_lowercase();
        for process in sys.processes().values() {
            if matches_process_name(&check_name, process.name(), process.exe(), process.cmd()) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_guardrail() {
        use sysinfo::{ProcessExt, SystemExt};

        let mut sys = sysinfo::System::new();
        sys.refresh_processes();
        let running_processes = sys
            .processes()
            .values()
            .map(|p| p.name().to_string())
            .collect::<Vec<_>>();

        if running_processes.is_empty() {
            return; // No processes found, skip test
        }

        // Pick the first running process to test
        let first_process = running_processes[0].clone();

        let guardrail = Process::new(&first_process);
        assert!(guardrail.check());
    }

    #[test]
    fn test_process_guardrail_not_exists() {
        let guardrail = Process::new("non_existent_process_123456789.exe");
        assert!(!guardrail.check());
    }

    #[test]
    fn test_process_guardrail_empty_name_fails_closed() {
        let guardrail = Process::new("");
        assert!(!guardrail.check());
    }

    #[test]
    fn test_matches_process_name_handles_linux_comm_truncation() {
        // Linux truncates /proc/<pid>/comm to 15 bytes, while exe and argv[0] preserve the full name.
        let long_name = "my_very_long_daemon_process_name";
        let truncated_comm = &long_name[..15];

        // Matches via exe basename when comm is truncated
        assert!(matches_process_name(
            long_name,
            truncated_comm,
            Path::new("/usr/local/bin/My_Very_Long_Daemon_Process_Name"),
            &[]
        ));

        // Matches via cmd[0] basename when exe is empty/unreadable
        assert!(matches_process_name(
            long_name,
            truncated_comm,
            Path::new(""),
            &[
                "/opt/bin/my_very_long_daemon_process_name".to_string(),
                "--flag".to_string()
            ]
        ));

        // Does not falsely match a different process sharing the 15-char prefix
        assert!(!matches_process_name(
            "my_very_long_daemon_other_name",
            truncated_comm,
            Path::new("/usr/local/bin/my_very_long_daemon_process_name"),
            &["/usr/local/bin/my_very_long_daemon_process_name".to_string()]
        ));
    }
}
