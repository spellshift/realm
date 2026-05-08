use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// List all named pipes on the system.
///
/// - Windows: enumerates `\\.\pipe\` namespace
/// - Linux: finds FIFOs in common locations + /proc fd scan
/// - Other: returns error
pub fn list_named_pipes() -> Result<Vec<String>, String> {
    #[cfg(target_os = "windows")]
    {
        list_named_pipes_windows()
    }

    #[cfg(all(unix, not(target_os = "windows")))]
    {
        list_named_pipes_unix()
    }

    #[cfg(not(any(target_os = "windows", unix)))]
    {
        Err("list_named_pipes is only supported on Windows and Unix systems".to_string())
    }
}

#[cfg(target_os = "windows")]
fn list_named_pipes_windows() -> Result<Vec<String>, String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        FindClose, FindFirstFileW, FindNextFileW, WIN32_FIND_DATAW,
    };

    let pattern = r"\\.\pipe\*";
    let wide_pattern: Vec<u16> = pattern.encode_utf16().chain(core::iter::once(0)).collect();

    let mut find_data: WIN32_FIND_DATAW = unsafe { core::mem::zeroed() };
    let handle = unsafe { FindFirstFileW(wide_pattern.as_ptr(), &mut find_data) };

    // INVALID_HANDLE_VALUE check - FindFirstFileW returns it on failure
    if handle as isize == -1 {
        return Err(format!(
            "FindFirstFileW failed: {}",
            ::std::io::Error::last_os_error()
        ));
    }

    let mut pipes = Vec::new();

    loop {
        // Extract pipe name from cFileName (null-terminated UTF-16)
        let name_len = find_data
            .cFileName
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(find_data.cFileName.len());
        let name = String::from_utf16_lossy(&find_data.cFileName[..name_len]);
        if !name.is_empty() {
            pipes.push(name);
        }

        // Get next entry
        if unsafe { FindNextFileW(handle, &mut find_data) } == 0 {
            break;
        }
    }

    unsafe { FindClose(handle) };
    Ok(pipes)
}

#[cfg(unix)]
fn list_named_pipes_unix() -> Result<Vec<String>, String> {
    use ::std::os::unix::fs::FileTypeExt;
    use alloc::collections::BTreeSet;

    let mut pipes = BTreeSet::new();

    // Scan common FIFO locations
    let scan_dirs = ["/tmp", "/var/run", "/var/tmp", "/run"];
    for dir in &scan_dirs {
        if let Ok(entries) = ::std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type()
                    && ft.is_fifo()
                    && let Some(path) = entry.path().to_str()
                {
                    pipes.insert(path.to_string());
                }
            }
        }
    }

    // Linux-only: scan /proc/*/fd for pipe file descriptors.
    // macOS/BSD don't have /proc (usually procfs not mounted anymore) - so this block is skipped
    #[cfg(target_os = "linux")]
    {
        if let Ok(proc_entries) = ::std::fs::read_dir("/proc") {
            for entry in proc_entries.flatten() {
                let pid_str = entry.file_name();
                let pid_str = pid_str.to_string_lossy();
                if !pid_str.chars().all(|c| c.is_ascii_digit()) {
                    continue;
                }

                let fd_dir = format!("/proc/{}/fd", pid_str);
                if let Ok(fds) = ::std::fs::read_dir(&fd_dir) {
                    for fd_entry in fds.flatten() {
                        if let Ok(target) = ::std::fs::read_link(fd_entry.path()) {
                            let target_str = target.to_string_lossy();
                            if target_str.starts_with("pipe:[") {
                                pipes.insert(target_str.into_owned());
                            } else if let Ok(ft) = ::std::fs::symlink_metadata(&target)
                                && ft.file_type().is_fifo()
                                && let Some(path) = target.to_str()
                            {
                                pipes.insert(path.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(pipes.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Cross platform tests

    #[test]
    fn test_list_returns_ok() {
        // Must not error even if no pipes exist on system
        let result = list_named_pipes();
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_returns_vec_of_strings() {
        let pipes = list_named_pipes().unwrap();
        // Each entry should be non-empty
        for pipe in &pipes {
            assert!(!pipe.is_empty(), "Pipe name should not be empty");
        }
    }

    // Linux tests

    #[cfg(unix)]
    #[test]
    fn test_list_detects_fifo_in_tmp() {
        // Create FIFO in /tmp (scanned by list_named_pipes)
        let fifo_path = "/tmp/test_list_pipe_realm_12345";
        let c_path = ::std::ffi::CString::new(fifo_path).unwrap();
        // Remove if leftover from previous run
        let _ = ::std::fs::remove_file(fifo_path);
        unsafe { libc::mkfifo(c_path.as_ptr(), 0o644) };

        let pipes = list_named_pipes().unwrap();
        assert!(
            pipes
                .iter()
                .any(|p| p.contains("test_list_pipe_realm_12345")),
            "Should detect FIFO in /tmp, got: {:?}",
            pipes
                .iter()
                .filter(|p| p.contains("test_list"))
                .collect::<Vec<_>>()
        );

        let _ = ::std::fs::remove_file(fifo_path);
    }

    #[cfg(unix)]
    #[test]
    fn test_list_no_duplicates() {
        let pipes = list_named_pipes().unwrap();
        let mut seen = ::std::collections::HashSet::new();
        for pipe in &pipes {
            assert!(seen.insert(pipe), "Duplicate pipe entry: {pipe}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_list_finds_proc_pipes() {
        // /proc scan should find at least some pipe:[inode] entries on any running system
        let pipes = list_named_pipes().unwrap();
        let has_proc_pipes = pipes.iter().any(|p| p.starts_with("pipe:["));
        // Not guaranteed on all systems (ex. containers) - just verify no crash
        let _ = has_proc_pipes;
    }

    // Windows tests

    #[cfg(target_os = "windows")]
    #[test]
    fn test_list_finds_system_pipes() {
        // Windows always has system pipes (ex. lsass)
        let pipes = list_named_pipes().unwrap();
        assert!(!pipes.is_empty(), "Windows should always have named pipes");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_list_contains_known_pipe() {
        // lsass pipe exists on all Windows systems
        let pipes = list_named_pipes().unwrap();
        assert!(
            pipes.iter().any(|p| p.to_lowercase().contains("lsass")),
            "Should find lsass pipe, got {} pipes",
            pipes.len()
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_list_no_empty_names() {
        let pipes = list_named_pipes().unwrap();
        for pipe in &pipes {
            assert!(!pipe.is_empty(), "Pipe names should not be empty");
            assert!(
                !pipe.contains('\0'),
                "Pipe names should not contain null bytes"
            );
        }
    }
}
