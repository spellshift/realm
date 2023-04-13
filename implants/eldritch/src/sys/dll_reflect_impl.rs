
use anyhow::Result;
use starlark::values::none::NoneType;
#[cfg(target_os = "windows")]
use windows_sys::Win32::{
    System::{
        Threading::{OpenProcess,PROCESS_ALL_ACCESS,CreateRemoteThread},
        LibraryLoader::{GetModuleHandleA, GetProcAddress},
        Memory::{VirtualAllocEx,MEM_RESERVE,MEM_COMMIT,PAGE_EXECUTE_READWRITE},
        Diagnostics::Debug::WriteProcessMemory,
    },
    Foundation::CloseHandle,
    Security::SECURITY_ATTRIBUTES
};
#[cfg(target_os = "windows")]
use std::ffi::c_void;

pub fn dll_reflect(dll_bytes: Vec<u32>, pid: u32) -> Result<NoneType> {
    if false { println!("Ignore unused vars dll_path: {:?}, pid: {}", dll_bytes, pid); }
    #[cfg(not(target_os = "windows"))]
    return Err(anyhow::anyhow!("This OS isn't supported by the dll_inject function.\nOnly windows systems are supported"));
    // Get the kernel32.dll base address
    unsafe {
        let h_kernel32 = GetModuleHandleA( "kernel32.dll\0".as_ptr() );

        // Get the address of the kernel function LoadLibraryA
        let _loadlibrary_function_ref = GetProcAddress(
            h_kernel32, 
            "LoadLibraryA\0".as_ptr()
        ).unwrap();
    }

    Ok(NoneType)
}

#[cfg(target_os = "windows")]
#[cfg(test)]
mod tests {
    use super::*;
    use core::time;
    use std::{process::Command, thread, path::Path, fs};
    use sysinfo::{Pid, Signal};
    use tempfile::NamedTempFile;
    use sysinfo::{ProcessExt,System,SystemExt,PidExt};
    
    #[test]
    fn test_dll_inject_simple() -> anyhow::Result<()>{
        const DLL_EXEC_WAIT_TIME: u64 = 5;
        // Get unique and unused temp file path
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap()).clone();
        tmp_file.close()?;

        // Get the path to our test dll file.
        let cargo_root = env!("CARGO_MANIFEST_DIR");
        let dll_bytes = include_bytes!("..\\..\\tests\\create_file_dll\\target\\debug\\create_file_dll.dll");

        // Out target process is notepad for stability and control.
        // The temp file is passed through an environment variable.
        let expected_process = Command::new("C:\\Windows\\System32\\notepad.exe").env("LIBTESTFILE", path.clone()).spawn();
        let target_pid = expected_process.unwrap().id();

        // Run our code.
        let _res = dll_reflect(dll_bytes, target_pid);

        let delay = time::Duration::from_secs(DLL_EXEC_WAIT_TIME);
        thread::sleep(delay);

        // Test that the test file was created
        let test_path = Path::new(path.as_str());
        assert!(test_path.is_file());

        // Delete test file
        let _ = fs::remove_file(test_path);
        
        // kill the target process notepad
        let mut sys = System::new();
        sys.refresh_processes();
        match sys.process(Pid::from_u32(target_pid)) {
            Some(res) => {
                res.kill_with(Signal::Kill);
            },
            None => {
            },
        }

        Ok(())
    }
}