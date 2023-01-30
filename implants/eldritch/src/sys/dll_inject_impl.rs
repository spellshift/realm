use anyhow::Result;
use starlark::values::none::NoneType;
use sysinfo::{ProcessExt,System,SystemExt,PidExt};


pub fn dll_inject(dll_path: String, pid: u32) -> Result<NoneType> {
    unimplemented!("Method unimplemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::time;
    use std::{process::Command, thread, path::Path, fs};
    use sysinfo::{Pid, Signal};

    fn find_first_process_of_name(process_name: String) -> Result<u32> {
        let mut sys = System::new();
        sys.refresh_processes();
        for (pid, process) in sys.processes() {
            if String::from(process.name()) == process_name {
                return Ok(pid.as_u32())
            }
        }
        return Err(anyhow::anyhow!(format!("No process of name {} found", process_name)));
    }
    
    #[test]
    fn test_dll_inject() -> anyhow::Result<()>{
        
        let test_dll_path = "C:\\Users\\Jack McKenna\\Documents\\test_dll\\target\\debug\\test_dll.dll".to_string();

        let expected_process = Command::new("C:\\Windows\\System32\\notepad.exe").spawn();
        let target_pid = expected_process.unwrap().id();

        let _res = dll_inject(test_dll_path, target_pid);
        
        let delay = time::Duration::from_secs(1);
        thread::sleep(delay);
        let test_path = Path::new("C:\\Users\\Jack McKenna\\Desktop\\win2.txt");

        assert!(test_path.is_file());

        let _ = fs::remove_file(test_path);
        Ok(())
    }

    #[test]
    fn test_find_first_process_of_name() -> anyhow::Result<()>{
        let process_name = "notepad.exe";
        let expected_process = Command::new("C:\\Windows\\System32\\notepad.exe").spawn();
        
        let expected_pid = expected_process.unwrap().id();
        let process_pid = find_first_process_of_name(process_name.to_string());

        let mut sys = System::new();
        sys.refresh_processes();
        if let Some(process) = sys.process(Pid::from_u32(expected_pid)) {
            process.kill_with(Signal::Kill);
        }
    
        assert_eq!(expected_pid, process_pid.unwrap());

        Ok(())
    }
}