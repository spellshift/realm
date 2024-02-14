
use windows_sys::Win32::{
        Foundation::{GetLastError, BOOL, HANDLE},
        System::Threading::{OpenProcess, PROCESS_ACCESS_RIGHTS},
    };

/// # Safety
///
/// Windows API: 
/// pub unsafe fn OpenProcess(dwdesiredaccess: PROCESS_ACCESS_RIGHTS, binherithandle: super::super::Foundation::BOOL, dwprocessid: u32) -> super::super::Foundation::HANDLE
pub unsafe fn open_process(
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
