use serde::{Deserialize, Serialize};

use crate::Guardrail;

#[derive(Serialize, Deserialize, Default)]
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
        for (_, process) in sys.processes() {
            if process.name().to_lowercase() == check_name {
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
}
