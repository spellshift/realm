use alloc::string::String;
use anyhow::{Result, anyhow};

#[cfg(target_os = "windows")]
use alloc::vec::Vec;
#[cfg(target_os = "windows")]
#[cfg(target_os = "windows")]
use windows_sys::Win32::NetworkManagement::NetManagement::{
    NERR_Success, NetUserSetInfo, USER_INFO_1003,
};

#[cfg(all(unix, not(target_os = "macos")))]
use nix::unistd::{Gid, Uid, chown};
#[cfg(all(unix, not(target_os = "macos")))]
use rand::{Rng, distributions::Alphanumeric, thread_rng};
#[cfg(all(unix, not(target_os = "macos")))]
use std::ffi::{CStr, CString};
#[cfg(all(unix, not(target_os = "macos")))]
use std::fs::{self, File};
#[cfg(all(unix, not(target_os = "macos")))]
use std::io::{BufRead, BufReader, Write};
#[cfg(all(unix, not(target_os = "macos")))]
use std::os::unix::fs::MetadataExt;
#[cfg(all(unix, not(target_os = "macos")))]
use std::path::Path;

pub fn change_user_password(username: String, password: String) -> Result<bool> {
    #[cfg(target_os = "windows")]
    {
        change_user_password_windows(username, password)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        change_user_password_linux(username, password)
    }
    #[cfg(target_os = "macos")]
    {
        let _ = username;
        let _ = password;
        // MacOS password changing requires OpenDirectory API or dscl.
        // Direct file manipulation is not standard.
        // Binaries are not allowed.
        // For now, we return an error.
        Err(anyhow!("Not supported on MacOS without system binaries"))
    }
    #[cfg(not(any(target_os = "windows", unix)))]
    {
        Err(anyhow!("Not supported on this OS"))
    }
}

#[cfg(target_os = "windows")]
fn change_user_password_windows(username: String, password: String) -> Result<bool> {
    let mut username_wide: Vec<u16> = username.encode_utf16().chain(Some(0)).collect();
    let mut password_wide: Vec<u16> = password.encode_utf16().chain(Some(0)).collect();

    let mut user_info = USER_INFO_1003 {
        usri1003_password: password_wide.as_mut_ptr(),
    };

    let result = unsafe {
        NetUserSetInfo(
            std::ptr::null(),
            username_wide.as_ptr(),
            1003,
            &mut user_info as *mut _ as *mut _,
            std::ptr::null_mut(),
        )
    };

    if result == NERR_Success {
        Ok(true)
    } else {
        Err(anyhow!("NetUserSetInfo failed with error code: {}", result))
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn change_user_password_linux(username: String, password: String) -> Result<bool> {
    // Generate salt for SHA-512 crypt
    let salt_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();
    let salt = format!("$6${}", salt_string);

    // Hash password
    let hash = crypt(&password, &salt)?;

    // Update /etc/shadow
    let path = Path::new("/etc/shadow");
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut new_lines = Vec::new();
    let mut found = false;

    for line in reader.lines() {
        let line = line?;
        if line.starts_with(&format!("{}:", username)) {
            // format: username:password:last:min:max:warn:inactive:expire:reserved
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                let mut new_parts: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
                new_parts[1] = hash.clone();
                new_lines.push(new_parts.join(":"));
                found = true;
                continue;
            }
        }
        new_lines.push(line);
    }

    if !found {
        return Err(anyhow!("User {} not found in /etc/shadow", username));
    }

    // Write to temp file and rename
    let mut temp_file = tempfile::NamedTempFile::new_in("/etc")?;

    // Preserve permissions and ownership from original shadow file
    let metadata = fs::metadata(path)?;
    fs::set_permissions(temp_file.path(), metadata.permissions())?;

    // Attempt to set ownership, ignore error if not possible (e.g. not root)
    // But modifying /etc/shadow usually requires root.
    let _ = chown(
        temp_file.path(),
        Some(Uid::from_raw(metadata.uid())),
        Some(Gid::from_raw(metadata.gid())),
    );

    for line in new_lines {
        writeln!(temp_file, "{}", line)?;
    }

    // Atomically replace /etc/shadow
    temp_file
        .persist(path)
        .map_err(|e| anyhow!("Failed to persist shadow file: {}", e))?;

    Ok(true)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn crypt(password: &str, salt: &str) -> Result<String> {
    // Check for libc crypt function.
    // Assuming libc is linked and available.
    // NOTE: Linking is handled in build.rs for linux targets
    unsafe extern "C" {
        fn crypt(key: *const libc::c_char, salt: *const libc::c_char) -> *mut libc::c_char;
    }

    let c_password = CString::new(password)?;
    let c_salt = CString::new(salt)?;

    unsafe {
        let res = crypt(c_password.as_ptr(), c_salt.as_ptr());
        if res.is_null() {
            return Err(anyhow!("crypt function failed"));
        }
        let res_str = CStr::from_ptr(res);
        Ok(res_str.to_string_lossy().into_owned())
    }
}
