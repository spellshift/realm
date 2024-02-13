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
// pub unsafe fn OpenProcess(dwdesiredaccess: PROCESS_ACCESS_RIGHTS, binherithandle: super::super::Foundation::BOOL, dwprocessid: u32) -> super::super::Foundation::HANDLE
fn open_process(
    dwdesiredaccess: PROCESS_ACCESS_RIGHTS,
    binherithandle: BOOL,
    dwprocessid: u32,
) -> anyhow::Result<HANDLE> {
    let process_handle: HANDLE =
        unsafe { OpenProcess(dwdesiredaccess, binherithandle, dwprocessid) };
    if process_handle == 0 {
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
