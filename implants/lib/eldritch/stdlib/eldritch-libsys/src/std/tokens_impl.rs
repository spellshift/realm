use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::Value;

#[cfg(target_os = "windows")]
#[derive(Debug)]
pub struct TokenEntry {
    pub id: i64,
    pub handle: isize,
    pub source: String,
    pub active: bool,
}

#[cfg(target_os = "windows")]
pub static TOKEN_STORE: std::sync::Mutex<Vec<TokenEntry>> = std::sync::Mutex::new(Vec::new());

#[cfg(target_os = "windows")]
static NEXT_TOKEN_ID: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(1);

#[cfg(target_os = "windows")]
fn next_id() -> i64 {
    NEXT_TOKEN_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[cfg(target_os = "windows")]
pub fn store_token(handle: isize, source: String) -> i64 {
    let id = next_id();
    if let Ok(mut store) = TOKEN_STORE.lock() {
        for entry in store.iter_mut() {
            entry.active = false;
        }
        store.push(TokenEntry {
            id,
            handle,
            source,
            active: true,
        });
    }
    id
}

#[cfg(target_os = "windows")]
pub fn get_active_token_handle() -> Option<isize> {
    if let Ok(store) = TOKEN_STORE.lock() {
        store.iter().find(|e| e.active).map(|e| e.handle)
    } else {
        None
    }
}

// apply active token to current thread (called from task thread startup)
#[cfg(target_os = "windows")]
pub fn apply_stored_token() {
    if let Some(handle) = get_active_token_handle() {
        unsafe {
            windows_sys::Win32::Security::ImpersonateLoggedOnUser(handle as *mut std::ffi::c_void);
        }
    }
}

// check if a named privilege is held and enabled on process token
#[cfg(target_os = "windows")]
pub fn check_privilege(privilege: &str) -> Result<bool, String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Security::{
        GetTokenInformation, LookupPrivilegeValueW, SE_PRIVILEGE_ENABLED, TOKEN_PRIVILEGES,
        TOKEN_QUERY, TokenPrivileges,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let mut token_handle = std::ptr::null_mut();
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle) } == 0 {
        return Err("OpenProcessToken failed".to_string());
    }

    let priv_wide: Vec<u16> = privilege
        .encode_utf16()
        .chain(core::iter::once(0))
        .collect();
    let mut target_luid = windows_sys::Win32::Foundation::LUID {
        LowPart: 0,
        HighPart: 0,
    };
    if unsafe { LookupPrivilegeValueW(std::ptr::null(), priv_wide.as_ptr(), &mut target_luid) } == 0
    {
        unsafe { CloseHandle(token_handle) };
        return Err(format!("Unknown privilege: {}", privilege));
    }

    let mut buf = vec![0u8; 4096];
    let mut ret_len: u32 = 0;
    if unsafe {
        GetTokenInformation(
            token_handle,
            TokenPrivileges,
            buf.as_mut_ptr() as *mut _,
            buf.len() as u32,
            &mut ret_len,
        )
    } == 0
    {
        unsafe { CloseHandle(token_handle) };
        return Err("GetTokenInformation failed".to_string());
    }
    unsafe { CloseHandle(token_handle) };

    let privs = unsafe { &*(buf.as_ptr() as *const TOKEN_PRIVILEGES) };
    let priv_slice = unsafe {
        core::slice::from_raw_parts(privs.Privileges.as_ptr(), privs.PrivilegeCount as usize)
    };

    for p in priv_slice {
        if p.Luid.LowPart == target_luid.LowPart && p.Luid.HighPart == target_luid.HighPart {
            return Ok((p.Attributes & SE_PRIVILEGE_ENABLED) != 0);
        }
    }
    Err(format!("Token does not hold {}", privilege))
}

pub fn tokens(pid: Option<i64>) -> Result<Vec<BTreeMap<String, Value>>, String> {
    match pid {
        None => list_stored(),
        Some(p) => list_process_token(p as u32),
    }
}

fn list_stored() -> Result<Vec<BTreeMap<String, Value>>, String> {
    #[cfg(target_os = "windows")]
    {
        let mut result = Vec::new();
        if let Ok(store) = TOKEN_STORE.lock() {
            for entry in store.iter() {
                let mut dict = BTreeMap::new();
                dict.insert("id".to_string(), Value::Int(entry.id));
                dict.insert("source".to_string(), Value::String(entry.source.clone()));
                dict.insert("active".to_string(), Value::Bool(entry.active));
                result.push(dict);
            }
        }
        Ok(result)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(Vec::new())
    }
}

fn list_process_token(pid: u32) -> Result<Vec<BTreeMap<String, Value>>, String> {
    #[cfg(target_os = "windows")]
    {
        list_process_token_windows(pid)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = pid;
        Err("tokens(pid) is only supported on Windows".to_string())
    }
}

#[cfg(target_os = "windows")]
fn list_process_token_windows(pid: u32) -> Result<Vec<BTreeMap<String, Value>>, String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Security::{
        GetTokenInformation, LookupAccountSidW, LookupPrivilegeNameW, SE_PRIVILEGE_ENABLED,
        TOKEN_PRIVILEGES, TOKEN_QUERY, TOKEN_USER, TokenPrivileges, TokenUser,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, OpenProcessToken, PROCESS_QUERY_INFORMATION,
    };

    let proc_handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid) };
    if proc_handle.is_null() {
        return Err(format!(
            "OpenProcess failed for PID {}: {}",
            pid,
            std::io::Error::last_os_error()
        ));
    }

    let mut token_handle = std::ptr::null_mut();
    if unsafe { OpenProcessToken(proc_handle, TOKEN_QUERY, &mut token_handle) } == 0 {
        unsafe { CloseHandle(proc_handle) };
        return Err(format!(
            "OpenProcessToken failed for PID {}: {}",
            pid,
            std::io::Error::last_os_error()
        ));
    }
    unsafe { CloseHandle(proc_handle) };

    let mut result = Vec::new();

    let mut user_buf = vec![0u8; 256];
    let mut ret_len: u32 = 0;
    if unsafe {
        GetTokenInformation(
            token_handle,
            TokenUser,
            user_buf.as_mut_ptr() as *mut _,
            user_buf.len() as u32,
            &mut ret_len,
        )
    } != 0
    {
        let token_user = unsafe { &*(user_buf.as_ptr() as *const TOKEN_USER) };
        let mut name_buf = [0u16; 256];
        let mut domain_buf = [0u16; 256];
        let mut name_len: u32 = 256;
        let mut domain_len: u32 = 256;
        let mut sid_use: i32 = 0;

        if unsafe {
            LookupAccountSidW(
                std::ptr::null(),
                token_user.User.Sid,
                name_buf.as_mut_ptr(),
                &mut name_len,
                domain_buf.as_mut_ptr(),
                &mut domain_len,
                &mut sid_use,
            )
        } != 0
        {
            let username = String::from_utf16_lossy(&name_buf[..name_len as usize]);
            let domain = String::from_utf16_lossy(&domain_buf[..domain_len as usize]);

            let mut info = BTreeMap::new();
            info.insert(
                "user".to_string(),
                Value::String(format!("{}\\{}", domain, username)),
            );
            info.insert("pid".to_string(), Value::Int(pid as i64));

            let mut priv_buf = vec![0u8; 4096];
            let mut priv_ret_len: u32 = 0;
            if unsafe {
                GetTokenInformation(
                    token_handle,
                    TokenPrivileges,
                    priv_buf.as_mut_ptr() as *mut _,
                    priv_buf.len() as u32,
                    &mut priv_ret_len,
                )
            } != 0
            {
                let privs = unsafe { &*(priv_buf.as_ptr() as *const TOKEN_PRIVILEGES) };
                let priv_slice = unsafe {
                    core::slice::from_raw_parts(
                        privs.Privileges.as_ptr(),
                        privs.PrivilegeCount as usize,
                    )
                };

                let mut priv_list = Vec::new();
                for p in priv_slice {
                    let mut priv_name = [0u16; 256];
                    let mut priv_name_len: u32 = 256;
                    if unsafe {
                        LookupPrivilegeNameW(
                            std::ptr::null(),
                            &p.Luid,
                            priv_name.as_mut_ptr(),
                            &mut priv_name_len,
                        )
                    } != 0
                    {
                        let name = String::from_utf16_lossy(&priv_name[..priv_name_len as usize]);
                        let status = if (p.Attributes & SE_PRIVILEGE_ENABLED) != 0 {
                            "enabled"
                        } else {
                            "disabled"
                        };
                        priv_list.push(Value::String(format!("{}={}", name, status)));
                    }
                }
                info.insert(
                    "privileges".to_string(),
                    Value::List(std::sync::Arc::new(spin::RwLock::new(priv_list))),
                );
            }
            result.push(info);
        }
    }

    unsafe { CloseHandle(token_handle) };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokens_no_args() {
        let result = tokens(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tokens_invalid_pid() {
        let result = tokens(Some(99999999));
        #[cfg(target_os = "windows")]
        assert!(result.is_err());
        #[cfg(not(target_os = "windows"))]
        assert!(result.is_err());
    }
}
