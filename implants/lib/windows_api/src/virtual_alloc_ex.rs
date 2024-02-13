
use {
    std::os::raw::c_void,
    windows_sys::Win32::{
        Foundation::{GetLastError, HANDLE},
        System::Memory::{
                VirtualAllocEx,
                PAGE_PROTECTION_FLAGS, VIRTUAL_ALLOCATION_TYPE,
            },
    },
};

/// # Safety
///
/// Windows API: 
/// pub unsafe fn VirtualAllocEx(hprocess: super::super::Foundation::HANDLE, lpaddress: *const ::core::ffi::c_void, dwsize: usize, flallocationtype: VIRTUAL_ALLOCATION_TYPE, flprotect: PAGE_PROTECTION_FLAGS) -> *mut ::core::ffi::c_void
pub unsafe fn virtual_alloc_ex(
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
