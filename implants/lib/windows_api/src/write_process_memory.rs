
use {
    std::{os::raw::c_void},
    windows_sys::Win32::{
        Foundation::{GetLastError, FALSE, HANDLE},
        System::{
            Diagnostics::Debug::WriteProcessMemory,
        },
    },
};
// pub unsafe fn WriteProcessMemory(hprocess: super::super::super::Foundation::HANDLE, lpbaseaddress: *const ::core::ffi::c_void, lpbuffer: *const ::core::ffi::c_void, nsize: usize, lpnumberofbyteswritten: *mut usize) -> super::super::super::Foundation::BOOL
pub unsafe fn write_process_memory(
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
