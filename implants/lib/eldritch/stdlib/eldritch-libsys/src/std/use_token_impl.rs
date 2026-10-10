use alloc::format;
use alloc::string::String;

// activate a stored token by ID
pub fn use_token(id: i64) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        if id == 0 {
            return revert_to_base();
        }
        activate_token(id)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = id;
        Err("use_token is only supported on Windows".to_string())
    }
}

#[cfg(target_os = "windows")]
fn revert_to_base() -> Result<bool, String> {
    if unsafe { windows_sys::Win32::Security::RevertToSelf() } == 0 {
        return Err(format!(
            "RevertToSelf failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    deactivate_all();
    Ok(true)
}

// mark all tokens in store as inactive
#[cfg(target_os = "windows")]
pub fn deactivate_all() {
    use super::tokens_impl::TOKEN_STORE;
    if let Ok(mut store) = TOKEN_STORE.lock() {
        for entry in store.iter_mut() {
            entry.active = false;
        }
    }
}

#[cfg(target_os = "windows")]
fn activate_token(id: i64) -> Result<bool, String> {
    use super::tokens_impl::TOKEN_STORE;

    if let Ok(mut store) = TOKEN_STORE.lock() {
        if !store.iter().any(|e| e.id == id) {
            return Err(format!("No token with id {} in store", id));
        }
        for entry in store.iter_mut() {
            entry.active = entry.id == id;
        }
        let handle = store.iter().find(|e| e.id == id).unwrap().handle;
        if unsafe {
            windows_sys::Win32::Security::ImpersonateLoggedOnUser(handle as *mut std::ffi::c_void)
        } == 0
        {
            return Err(format!(
                "ImpersonateLoggedOnUser failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(true)
    } else {
        Err("Failed to lock token store".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_token_invalid_id() {
        let result = use_token(9999);
        assert!(result.is_err());
    }
}
