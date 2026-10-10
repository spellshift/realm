use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_core::Value;
use spin::RwLock;

fn make_list(items: Vec<Value>) -> Value {
    Value::List(Arc::new(RwLock::new(items)))
}

fn make_dict(map: BTreeMap<String, Value>) -> Value {
    Value::Dictionary(Arc::new(RwLock::new(
        map.into_iter()
            .map(|(k, v)| (Value::String(k), v))
            .collect(),
    )))
}

/// List all named pipes on the system
/// If detailed=true (Windows only), returns List<Dict> with name/instances/max_instances
/// Otherwise returns List<str> of pipe names.
pub fn list_named_pipes(detailed: Option<bool>) -> Result<Value, String> {
    let detailed = detailed.unwrap_or(false);

    #[cfg(target_os = "windows")]
    {
        if detailed {
            list_named_pipes_windows_detailed().map(make_list)
        } else {
            list_named_pipes_windows()
                .map(|v| make_list(v.into_iter().map(Value::String).collect()))
        }
    }

    #[cfg(all(unix, not(target_os = "windows")))]
    {
        let _ = detailed; // detailed not supported on unix
        list_named_pipes_unix().map(|v| make_list(v.into_iter().map(Value::String).collect()))
    }

    #[cfg(not(any(target_os = "windows", unix)))]
    {
        let _ = detailed;
        Err("list_named_pipes is only supported on Windows and Unix systems".to_string())
    }
}

/// Enumerate pipe names via FindFirstFileW/FindNextFileW
#[cfg(target_os = "windows")]
fn list_named_pipes_windows() -> Result<Vec<String>, String> {
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

/// Enumerate pipes with instance info.
/// Uses GetNamedPipeInfo (max instances) and GetNamedPipeHandleStateW (current instances)
#[cfg(target_os = "windows")]
fn list_named_pipes_windows_detailed() -> Result<Vec<Value>, String> {
    use ::std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::Pipes::{GetNamedPipeHandleStateW, GetNamedPipeInfo};

    let names = list_named_pipes_windows()?;
    let mut results = Vec::new();

    for name in &names {
        let pipe_path = format!(r"\\.\pipe\{}", name);
        let mut info = BTreeMap::new();
        info.insert("name".to_string(), Value::String(name.clone()));

        match ::std::fs::OpenOptions::new().read(true).open(&pipe_path) {
            Ok(file) => {
                let handle = file.as_raw_handle();

                // Current instances via GetNamedPipeHandleStateW
                let mut cur_inst: u32 = 0;
                unsafe {
                    GetNamedPipeHandleStateW(
                        handle,
                        core::ptr::null_mut(), // state
                        &mut cur_inst,
                        core::ptr::null_mut(), // max collection count
                        core::ptr::null_mut(), // collect data timeout
                        core::ptr::null_mut(), // username
                        0,                     // username max size
                    );
                };
                info.insert("instances".to_string(), Value::Int(cur_inst as i64));

                // Max instances via GetNamedPipeInfo
                let mut max_inst: u32 = 0;
                let ok = unsafe {
                    GetNamedPipeInfo(
                        handle,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        &mut max_inst,
                    )
                };
                if ok != 0 {
                    // 255 = PIPE_UNLIMITED_INSTANCES
                    if max_inst == 255 {
                        info.insert(
                            "max_instances".to_string(),
                            Value::String("UNLIMITED".to_string()),
                        );
                    } else {
                        info.insert("max_instances".to_string(), Value::Int(max_inst as i64));
                    }
                } else {
                    info.insert(
                        "max_instances".to_string(),
                        Value::String("UNKNOWN".to_string()),
                    );
                }
            }
            Err(_) => {
                info.insert(
                    "instances".to_string(),
                    Value::String("ACCESS_DENIED".to_string()),
                );
                info.insert(
                    "max_instances".to_string(),
                    Value::String("ACCESS_DENIED".to_string()),
                );
            }
        }

        results.push(make_dict(info));
    }

    Ok(results)
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

    fn unwrap_list(val: Value) -> Vec<Value> {
        if let Value::List(arc) = val {
            arc.read().clone()
        } else {
            panic!("Expected List value");
        }
    }

    #[test]
    fn test_list_returns_ok() {
        // Must not error even if no pipes exist on system (somehow?)
        assert!(list_named_pipes(None).is_ok());
    }

    #[test]
    fn test_list_returns_list() {
        let _ = unwrap_list(list_named_pipes(None).unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn test_list_detects_fifo_in_tmp() {
        // Create FIFO in /tmp (scanned by list_named_pipes)
        let fifo_path = "/tmp/test_list_pipe_realm_12345";
        let c_path = ::std::ffi::CString::new(fifo_path).unwrap();
        // Remove if leftover from previous run
        let _ = ::std::fs::remove_file(fifo_path);
        unsafe { libc::mkfifo(c_path.as_ptr(), 0o644) };

        let pipes = unwrap_list(list_named_pipes(None).unwrap());
        assert!(
            pipes
                .iter()
                .any(|p| matches!(p, Value::String(s) if s.contains("test_list_pipe_realm_12345"))),
            "Should detect FIFO in /tmp"
        );
        let _ = ::std::fs::remove_file(fifo_path);
    }

    #[cfg(unix)]
    #[test]
    fn test_list_no_duplicates() {
        let pipes = unwrap_list(list_named_pipes(None).unwrap());
        let mut seen = ::std::collections::HashSet::new();
        for pipe in &pipes {
            if let Value::String(s) = pipe {
                assert!(seen.insert(s.clone()), "Duplicate: {s}");
            }
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_list_finds_system_pipes() {
        // Windows always has system pipes (ex. lsass)
        let pipes = unwrap_list(list_named_pipes(None).unwrap());
        assert!(!pipes.is_empty(), "Windows should always have named pipes");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_list_detailed_returns_dicts() {
        let pipes = unwrap_list(list_named_pipes(Some(true)).unwrap());
        assert!(!pipes.is_empty());
        if let Value::Dictionary(arc) = &pipes[0] {
            let info = arc.read();
            assert!(info.contains_key(&Value::String("name".to_string())));
            assert!(info.contains_key(&Value::String("instances".to_string())));
            assert!(info.contains_key(&Value::String("max_instances".to_string())));
        } else {
            panic!("Expected Dict in detailed mode");
        }
    }
}
