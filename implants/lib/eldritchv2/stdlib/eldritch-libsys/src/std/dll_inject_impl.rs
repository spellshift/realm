#[cfg(target_os = "windows")]
use alloc::format;
use alloc::string::String;
use anyhow::Result;

#[cfg(target_os = "windows")]
use std::ffi::c_void;
#[cfg(target_os = "windows")]
use windows_sys::Win32::{
    Foundation::CloseHandle,
    Security::SECURITY_ATTRIBUTES,
    System::{
        Diagnostics::Debug::WriteProcessMemory,
        LibraryLoader::{GetModuleHandleA, GetProcAddress},
        Memory::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, VirtualAllocEx},
        Threading::{CreateRemoteThread, OpenProcess, PROCESS_ALL_ACCESS},
    },
};

pub fn dll_inject(dll_path: String, pid: u32) -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        // Suppress unused variable warnings for non-Windows builds
        let _ = dll_path;
        let _ = pid;
    }

    #[cfg(not(target_os = "windows"))]
    return Err(anyhow::anyhow!(
        "This OS isn't supported by the dll_inject function.\nOnly windows systems are supported"
    ));
    #[cfg(target_os = "windows")]
    unsafe {
        let dll_path_null_terminated: String = format!("{}\0", dll_path);

        // Get the kernel32.dll base address
        let h_kernel32 = GetModuleHandleA("kernel32.dll\0".as_ptr());

        // Get the address of the kernel function LoadLibraryA
        let loadlibrary_function_ref =
            GetProcAddress(h_kernel32, "LoadLibraryA\0".as_ptr()).unwrap();

        // Open a handle to the remote process
        let target_process_memory_handle = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);

        // Allocate memory in the remote process that we'll copy the DLL path string to.
        let target_process_allocated_memory_handle = VirtualAllocEx(
            target_process_memory_handle,
            std::ptr::null::<c_void>(),
            dll_path_null_terminated.len() + 1,
            MEM_RESERVE | MEM_COMMIT,
            PAGE_EXECUTE_READWRITE,
        );

        // Write the DLL path into the remote processes newly allocated memory
        let _write_proccess_memory_res = WriteProcessMemory(
            target_process_memory_handle,
            target_process_allocated_memory_handle,
            dll_path_null_terminated.as_bytes().as_ptr() as *const c_void,
            dll_path_null_terminated.len(),
            std::ptr::null_mut::<usize>(),
        );

        // Kickoff our DLL in the remote process
        let _remote_thread_return_val = CreateRemoteThread(
            target_process_memory_handle,
            std::ptr::null::<SECURITY_ATTRIBUTES>(),
            0,
            Some(
                // Translate our existing function return to the one LoadLibraryA wants.
                std::mem::transmute::<
                    unsafe extern "system" fn() -> isize,
                    extern "system" fn(*mut c_void) -> u32,
                >(loadlibrary_function_ref),
            ),
            target_process_allocated_memory_handle,
            0,
            std::ptr::null_mut::<u32>(),
        );

        CloseHandle(target_process_memory_handle);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "windows")]
    use core::time;
    #[cfg(target_os = "windows")]
    use std::{fs, path::Path, process::Command, thread};
    #[cfg(target_os = "windows")]
    use sysinfo::{Pid, Signal};
    #[cfg(target_os = "windows")]
    use sysinfo::{PidExt, ProcessExt, System, SystemExt};
    #[cfg(target_os = "windows")]
    use tempfile::NamedTempFile;

    #[cfg(target_os = "windows")]
    #[test]
    fn test_dll_inject_simple() -> anyhow::Result<()> {
        const DLL_EXEC_WAIT_TIME: u64 = 5;
        // Get unique and unused temp file path
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap()).clone();
        tmp_file.close()?;

        // Get the path to our test dll file.
        let cargo_root = env!("CARGO_MANIFEST_DIR");
        let relative_path_to_test_dll =
            "..\\..\\..\\bin\\create_file_dll\\target\\debug\\create_file_dll.dll";
        let test_dll_path = Path::new(cargo_root).join(relative_path_to_test_dll);
        assert!(test_dll_path.is_file());

        // Out target process is notepad for stability and control.
        // The temp file is passed through an environment variable.
        let expected_process = Command::new("C:\\Windows\\System32\\notepad.exe")
            .env("LIBTESTFILE", path.clone())
            .spawn();
        let target_pid = expected_process.unwrap().id();

        // Run our code.
        let _res = dll_inject(test_dll_path.to_string_lossy().to_string(), target_pid);

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
        if let Some(res) = sys.process(Pid::from_u32(target_pid)) {
            res.kill_with(Signal::Kill);
        }

        Ok(())
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_dll_inject_non_windows() {
        let res = dll_inject("foo".to_string(), 123);
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .to_string()
                .contains("Only windows systems are supported")
        );
    }
}
