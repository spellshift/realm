use anyhow::Result;
use {
    object::LittleEndian as LE,
    object::{Object, ObjectSection},
    std::{os::raw::c_void, ptr::null_mut},
    windows_sys::Win32::Security::SECURITY_ATTRIBUTES,
    windows_sys::Win32::System::Threading::CreateRemoteThread,
    windows_sys::Win32::{
        Foundation::{GetLastError, BOOL, FALSE, HANDLE},
        System::{
            Diagnostics::Debug::WriteProcessMemory,
            Memory::{
                VirtualAllocEx, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
                PAGE_PROTECTION_FLAGS, VIRTUAL_ALLOCATION_TYPE,
            },
            Threading::{OpenProcess, PROCESS_ACCESS_RIGHTS, PROCESS_ALL_ACCESS},
        },
    },
};

// fn CreateRemoteThread(hprocess: isize, lpthreadattributes: *const SECURITY_ATTRIBUTES, dwstacksize: usize, lpstartaddress: Option<fn(*mut c_void) -> u32>, lpparameter: *const c_void, dwcreationflags: u32, lpthreadid: *mut u32) -> isize
pub unsafe fn create_remote_thread(
    hprocess: isize,
    lpthreadattributes: *const SECURITY_ATTRIBUTES,
    dwstacksize: usize,
    lpstartaddress: Option<*mut c_void>,
    lpparameter: *const c_void,
    dwcreationflags: u32,
    lpthreadid: *mut u32,
) -> anyhow::Result<isize> {
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
    if res == 0 {
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
