use alloc::format;
use alloc::string::String;

pub fn impersonate(pid: i64) -> Result<i64, String> {
    #[cfg(target_os = "windows")]
    {
        impersonate_windows(pid as u32)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = pid;
        Err("impersonate is only supported on Windows".to_string())
    }
}

#[cfg(target_os = "windows")]
fn impersonate_windows(pid: u32) -> Result<i64, String> {
    use super::tokens_impl::{check_privilege, store_token};
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Security::{
        DuplicateTokenEx, ImpersonateLoggedOnUser, SecurityImpersonation, TOKEN_DUPLICATE,
        TOKEN_IMPERSONATE, TOKEN_QUERY, TokenImpersonation,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, OpenProcessToken, PROCESS_QUERY_INFORMATION,
    };

    // check SeDebugPrivilege before OpenProcess
    match check_privilege("SeDebugPrivilege") {
        Ok(true) => {}
        Ok(false) => {
            return Err(
                "SeDebugPrivilege held but not enabled. Call sys.enable_privilege(\"SeDebugPrivilege\") first".to_string()
            );
        }
        Err(_) => {
            return Err(
                "Token does not hold SeDebugPrivilege. Cannot open other users' process tokens"
                    .to_string(),
            );
        }
    }

    let proc_handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid) };
    if proc_handle.is_null() {
        return Err(format!(
            "OpenProcess failed for PID {}: {}",
            pid,
            std::io::Error::last_os_error()
        ));
    }

    let mut token_handle = std::ptr::null_mut();
    if unsafe {
        OpenProcessToken(
            proc_handle,
            TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_IMPERSONATE,
            &mut token_handle,
        )
    } == 0
    {
        unsafe { CloseHandle(proc_handle) };
        return Err(format!(
            "OpenProcessToken failed for PID {}: {}",
            pid,
            std::io::Error::last_os_error()
        ));
    }
    unsafe { CloseHandle(proc_handle) };

    let mut dup_token = std::ptr::null_mut();
    if unsafe {
        DuplicateTokenEx(
            token_handle,
            TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_IMPERSONATE,
            std::ptr::null(),
            SecurityImpersonation,
            TokenImpersonation,
            &mut dup_token,
        )
    } == 0
    {
        unsafe { CloseHandle(token_handle) };
        return Err(format!(
            "DuplicateTokenEx failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    unsafe { CloseHandle(token_handle) };

    if unsafe { ImpersonateLoggedOnUser(dup_token) } == 0 {
        unsafe { CloseHandle(dup_token) };
        return Err(format!(
            "ImpersonateLoggedOnUser failed: {}",
            std::io::Error::last_os_error()
        ));
    }

    let source = format!("impersonate:pid:{}", pid);
    let id = store_token(dup_token as isize, source);

    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impersonate_invalid_pid() {
        let result = impersonate(99999999);
        assert!(result.is_err());
    }
}
