#[cfg(feature = "stdlib")]
use alloc::collections::BTreeMap;
#[cfg(feature = "stdlib")]
use alloc::string::String;

#[cfg(feature = "stdlib")]
pub fn get_extended_perms(
    path: String,
) -> Result<BTreeMap<String, String>, String> {
    get_extended_perms_impl(&path).map_err(|e| e.to_string())
}

#[cfg(not(feature = "stdlib"))]
pub fn get_extended_perms(
    _path: String,
) -> Result<alloc::collections::BTreeMap<alloc::string::String, alloc::string::String>, alloc::string::String> {
    Err("get_extended_perms requires stdlib feature".into())
}

#[cfg(feature = "stdlib")]
fn get_extended_perms_impl(path: &str) -> Result<BTreeMap<String, String>, String> {
    use std::fs;

    // Verify path exists
    fs::metadata(path).map_err(|e| format!("Cannot access '{}': {}", path, e))?;

    let mut result = BTreeMap::new();

    #[cfg(target_os = "linux")]
    {
        // Get extended attributes (lsattr equivalent)
        if let Ok(attrs) = get_file_attrs(path) {
            result.insert("attrs".to_string(), attrs);
        }

        // Get SELinux context if available
        if let Ok(secontext) = get_selinux_context(path) {
            result.insert("secontext".to_string(), secontext);
        }

        // Get POSIX ACLs (getfacl)
        if let Ok(facl) = get_posix_acl(path) {
            result.insert("facl".to_string(), facl);
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS doesn't have lsattr or SELinux by default
        // Could potentially get ACLs but let's keep it simple for now
    }

    #[cfg(windows)]
    {
        // Get Windows ACLs (icacls)
        if let Ok(icacls) = get_windows_acl(path) {
            result.insert("icacls".to_string(), icacls);
        }
    }

    Ok(result)
}

/// Get extended file attributes (equivalent to lsattr on Linux)
#[cfg(target_os = "linux")]
fn get_file_attrs(path: &str) -> Result<String, String> {
    use std::process::Command;

    // Try to use lsattr command
    let output = Command::new("lsattr")
        .arg("-d")  // Don't recurse into directories
        .arg(path)
        .output()
        .map_err(|e| format!("Failed to run lsattr: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse output: attributes are in first column, filename in second
        // Format: ----i--------e-- filename
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let attrs = parts[0];
                // Extract just the relevant flags (remove trailing filename)
                return Ok(attrs.to_string());
            }
        }
        // If we got here, parsing might have failed
        return Ok(stdout.trim().to_string());
    }

    // Fall back to using ioctl directly
    get_attrs_via_ioctl(path)
}

/// Get file attributes using ioctl (FS_IOC_GETFLAGS)
#[cfg(target_os = "linux")]
fn get_attrs_via_ioctl(path: &str) -> Result<String, String> {
    use std::fs::File;
    use std::os::unix::io::AsRawFd;

    // Linux ioctl constants
    const FS_IOC_GETFLAGS: u32 = 0x80046601;
    const FS_IMMUTABLE_FL: u32 = 0x00000010;
    const FS_APPEND_FL: u32 = 0x00000004;
    const FS_COMPR_FL: u32 = 0x00000020;
    const FS_SYNC_FL: u32 = 0x00000008;
    const FS_NOATIME_FL: u32 = 0x00004000;
    const FS_DIRSYNC_FL: u32 = 0x00010000;

    let file = File::open(path).map_err(|e| format!("Failed to open '{}': {}", path, e))?;
    let fd = file.as_raw_fd();

    let mut flags: u32 = 0;
    let ret = unsafe {
        libc::ioctl(fd, FS_IOC_GETFLAGS as libc::c_ulong, &mut flags)
    };

    if ret < 0 {
        return Err(format!(
            "ioctl FS_IOC_GETFLAGS failed: {}",
            std::io::Error::last_os_error()
        ));
    }

    // Build attrs string similar to lsattr output
    let mut attrs = String::new();
    attrs.push(if flags & FS_APPEND_FL != 0 { 'a' } else { '-' });
    attrs.push(if flags & FS_IMMUTABLE_FL != 0 { 'i' } else { '-' });
    attrs.push(if flags & FS_COMPR_FL != 0 { 'c' } else { '-' });
    attrs.push(if flags & FS_SYNC_FL != 0 { 's' } else { '-' });
    attrs.push(if flags & FS_NOATIME_FL != 0 { 'A' } else { '-' });
    attrs.push(if flags & FS_DIRSYNC_FL != 0 { 'D' } else { '-' });
    attrs.push_str("--------e--"); // Remaining flags simplified

    Ok(attrs)
}

/// Get SELinux security context
#[cfg(target_os = "linux")]
fn get_selinux_context(path: &str) -> Result<String, String> {
    use std::process::Command;

    // Try getfattr first for xattr security.selinux
    let output = Command::new("getfattr")
        .arg("-n")
        .arg("security.selinux")
        .arg("--absolute-names")
        .arg(path)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse: # file: /path\nsecurity.selinux="context"
            for line in stdout.lines() {
                if line.starts_with("security.selinux=") {
                    let context = line
                        .trim_start_matches("security.selinux=")
                        .trim_matches('"')
                        .to_string();
                    return Ok(context);
                }
            }
        }
    }

    // Fallback to matchpathcon
    let output = Command::new("matchpathcon")
        .arg(path)
        .output()
        .map_err(|e| format!("SELinux not available: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let context = stdout.trim();
        if !context.is_empty() && context != "<???:???>" {
            return Ok(context.to_string());
        }
    }

    Err("SELinux context not available".to_string())
}

/// Get POSIX ACLs (getfacl equivalent)
#[cfg(target_os = "linux")]
fn get_posix_acl(path: &str) -> Result<String, String> {
    use std::process::Command;

    let output = Command::new("getfacl")
        .arg("--absolute-names")
        .arg(path)
        .output()
        .map_err(|e| format!("Failed to run getfacl: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("getfacl failed: {}", stderr.trim()))
    }
}

/// Get Windows ACLs (icacls equivalent)
#[cfg(windows)]
fn get_windows_acl(path: &str) -> Result<String, String> {
    use std::process::Command;

    let output = Command::new("icacls")
        .arg(path)
        .output()
        .map_err(|e| format!("Failed to run icacls: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("icacls failed: {}", stderr.trim()))
    }
}

#[cfg(test)]
#[cfg(feature = "stdlib")]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_extended_perms_basic() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        let result = get_extended_perms(path).unwrap();
        // Should return a map (may be empty or contain platform-specific info)
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_get_extended_perms_nonexistent() {
        let result = get_extended_perms("/nonexistent/path".to_string());
        assert!(result.is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_get_file_attrs_ioctl() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        // Note: FS_IOC_GETFLAGS may fail on tmpfs or other filesystems
        // that don't support extended attributes
        let result = get_attrs_via_ioctl(&path);
        
        // Either succeeds with attrs string, or fails with appropriate error
        match result {
            Ok(attrs) => {
                assert!(!attrs.is_empty());
                assert!(attrs.len() >= 2);
            }
            Err(e) => {
                // ENOTTY (25) is expected on filesystems that don't support ioctl
                assert!(
                    e.contains("Inappropriate ioctl for device") || e.contains("ioctl"),
                    "Unexpected error: {}",
                    e
                );
            }
        }
    }
}
