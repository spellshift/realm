use anyhow::Result;
use alloc::string::String;
#[cfg(target_os = "windows")]
use alloc::format;

#[cfg(target_os = "windows")]
use std::ffi::c_void;
#[cfg(target_os = "windows")]
use windows_sys::Win32::{
    Foundation::CloseHandle,
    Security::SECURITY_ATTRIBUTES,
    System::{
        Diagnostics::Debug::WriteProcessMemory,
        LibraryLoader::{GetModuleHandleA, GetProcAddress},
        Memory::{VirtualAllocEx, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE},
        Threading::{CreateRemoteThread, OpenProcess, PROCESS_ALL_ACCESS},
    },
};

pub fn dll_inject(dll_path: String, pid: u32) -> Result<()> {
    #[allow(unused_variables)]
    let dll_path = dll_path;
    #[allow(unused_variables)]
    let pid = pid;

    if false {
        // println!("Ignore unused vars dll_path: {}, pid: {}", dll_path, pid);
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
