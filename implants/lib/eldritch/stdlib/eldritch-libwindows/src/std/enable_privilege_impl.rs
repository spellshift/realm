use alloc::format;
use alloc::string::String;

pub fn enable_privilege(privilege: String) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        enable_privilege_windows(&privilege)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = privilege;
        Err("enable_privilege is only supported on Windows".to_string())
    }
}

#[cfg(target_os = "windows")]
fn enable_privilege_windows(privilege: &str) -> Result<bool, String> {
    use windows_sys::Win32::Foundation::{CloseHandle, ERROR_NOT_ALL_ASSIGNED};
    use windows_sys::Win32::Security::{
        AdjustTokenPrivileges, LookupPrivilegeValueW, SE_PRIVILEGE_ENABLED,
        TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    // open current process token
    let mut token_handle = std::ptr::null_mut();
    if unsafe {
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token_handle,
        )
    } == 0
    {
        return Err(format!(
            "OpenProcessToken failed: {}",
            std::io::Error::last_os_error()
        ));
    }

    // lookup privilege LUID
    let priv_wide: Vec<u16> = privilege
        .encode_utf16()
        .chain(core::iter::once(0))
        .collect();
    let mut luid = windows_sys::Win32::Foundation::LUID {
        LowPart: 0,
        HighPart: 0,
    };
    if unsafe { LookupPrivilegeValueW(std::ptr::null(), priv_wide.as_ptr(), &mut luid) } == 0 {
        unsafe { CloseHandle(token_handle) };
        return Err(format!(
            "LookupPrivilegeValue failed for '{}': {}",
            privilege,
            std::io::Error::last_os_error()
        ));
    }

    // build TOKEN_PRIVILEGES
    let mut tp: TOKEN_PRIVILEGES = unsafe { core::mem::zeroed() };
    tp.PrivilegeCount = 1;
    tp.Privileges[0].Luid = luid;
    tp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

    // adjust token
    let result = unsafe {
        AdjustTokenPrivileges(
            token_handle,
            0,
            &tp,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };

    let last_err = std::io::Error::last_os_error();
    unsafe { CloseHandle(token_handle) };

    if result == 0 {
        return Err(format!("AdjustTokenPrivileges failed: {}", last_err));
    }

    if last_err.raw_os_error() == Some(ERROR_NOT_ALL_ASSIGNED as i32) {
        return Err(format!(
            "Token does not hold privilege '{}' - cannot enable",
            privilege
        ));
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enable_privilege_nonexistent() {
        let result = enable_privilege("SeFunnyManPrivilege".to_string());
        #[cfg(target_os = "windows")]
        assert!(result.is_err());
        #[cfg(not(target_os = "windows"))]
        assert!(result.is_err());
    }

    #[test]
    fn test_enable_privilege_empty_name() {
        let result = enable_privilege("".to_string());
        assert!(result.is_err());
    }
}
