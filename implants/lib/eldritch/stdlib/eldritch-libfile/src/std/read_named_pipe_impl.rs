use alloc::format;
use alloc::string::String;
use alloc::vec;

/// Read data from a named pipe with optional timeout.
///
/// - Windows: opens `\\.\pipe\<name>`
/// - Linux: opens FIFO at `<name>` (must be full path)
/// - Other: returns error
pub fn read_named_pipe(name: String, timeout: Option<i64>) -> Result<String, String> {
    let timeout_secs = timeout.unwrap_or(5);

    #[cfg(target_os = "windows")]
    {
        read_named_pipe_windows(&name, timeout_secs)
    }

    #[cfg(all(unix, not(target_os = "windows")))]
    {
        read_named_pipe_unix(&name, timeout_secs)
    }

    #[cfg(not(any(target_os = "windows", unix)))]
    {
        let _ = (name, timeout_secs);
        Err("read_named_pipe is only supported on Windows, Linux, macOS, and BSD".to_string())
    }
}

#[cfg(target_os = "windows")]
fn read_named_pipe_windows(name: &str, timeout_secs: i64) -> Result<String, String> {
    use ::std::io::Read;
    use ::std::os::windows::io::{AsRawHandle, RawHandle};
    use windows_sys::Win32::System::Pipes::PeekNamedPipe;

    // Build pipe path: \\.\pipe\<name>
    let pipe_path = if name.starts_with(r"\\") {
        name.to_string()
    } else {
        format!(r"\\.\pipe\{}", name)
    };

    // Open pipe using std::fs (basically a CreateFileW wrapper, handles that automatically)
    let file = ::std::fs::OpenOptions::new()
        .read(true)
        .open(&pipe_path)
        .map_err(|e| format!("Failed to open pipe '{}': {}", pipe_path, e))?;

    let handle = file.as_raw_handle();

    // Poll with PeekNamedPipe + timeout instead of blocking read().
    // read() on named pipe blocks forever when there's no data (it just hangs imix :( )
    let deadline = ::std::time::Instant::now()
        + ::std::time::Duration::from_secs(if timeout_secs <= 0 {
            0
        } else {
            timeout_secs as u64
        });

    let mut result = Vec::new();
    let mut buf = vec![0u8; 4096];

    loop {
        // Check if data available without blocking
        let mut avail: u32 = 0;
        let peek_ok = unsafe {
            PeekNamedPipe(
                handle,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                &mut avail,
                core::ptr::null_mut(),
            )
        };

        if peek_ok == 0 {
            // Pipe broken or closed
            break;
        }

        if avail > 0 {
            // Data available - use std::io::Read (won't block since we know bytes are available)
            let read_size = core::cmp::min(avail as usize, buf.len());
            match (&file).read(&mut buf[..read_size]) {
                Ok(0) => break,
                Ok(n) => result.extend_from_slice(&buf[..n]),
                Err(_) => break,
            }
            continue;
        }

        // No data - check timeout
        if timeout_secs <= 0 || ::std::time::Instant::now() >= deadline {
            break;
        }

        // Brief sleep before polling again
        ::std::thread::sleep(::std::time::Duration::from_millis(50));
    }

    // file dropped here - handle closed automatically

    if result.is_empty() {
        return Err(format!(
            "No data read from pipe '{}' within {}s",
            pipe_path, timeout_secs
        ));
    }

    String::from_utf8(result).map_err(|e| format!("Pipe data is not valid UTF-8: {}", e))
}

#[cfg(unix)]
fn read_named_pipe_unix(name: &str, timeout_secs: i64) -> Result<String, String> {
    use ::std::os::unix::fs::OpenOptionsExt;
    use ::std::os::unix::io::AsRawFd;

    // Open FIFO in non-blocking mode first (prevents blocking on open if no writer)
    let file = ::std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(name)
        .map_err(|e| format!("Failed to open pipe '{}': {}", name, e))?;

    let fd = file.as_raw_fd();

    // Use poll() with timeout to wait for data
    let timeout_ms = if timeout_secs <= 0 {
        0
    } else {
        (timeout_secs * 1000) as i32
    };
    let mut pollfd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };

    let poll_result = unsafe { libc::poll(&mut pollfd, 1, timeout_ms) };
    if poll_result == 0 {
        return Err(format!(
            "Pipe '{}' read timed out after {}s",
            name, timeout_secs
        ));
    }
    if poll_result < 0 {
        return Err(format!(
            "poll() failed on pipe '{}': {}",
            name,
            ::std::io::Error::last_os_error()
        ));
    }

    // Data available - read it
    let mut result = Vec::new();
    let mut buf = vec![0u8; 4096];
    loop {
        let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        if n > 0 {
            result.extend_from_slice(&buf[..n as usize]);
        }
        if n <= 0 {
            break;
        }
    }

    String::from_utf8(result).map_err(|e| format!("Pipe data is not valid UTF-8: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn test_read_named_pipe_linux_fifo() {
        use ::std::io::Write;

        let dir = tempfile::tempdir().unwrap();
        let fifo_path = dir.path().join("test_pipe");
        let fifo_str = fifo_path.to_str().unwrap().to_string();

        // Create FIFO
        let c_path = ::std::ffi::CString::new(fifo_str.clone()).unwrap();
        unsafe { libc::mkfifo(c_path.as_ptr(), 0o644) };

        // Write from another thread
        let fifo_str_clone = fifo_str.clone();
        let writer = ::std::thread::spawn(move || {
            let mut f = ::std::fs::OpenOptions::new()
                .write(true)
                .open(&fifo_str_clone)
                .unwrap();
            f.write_all(b"hello from pipe").unwrap();
        });

        let result = read_named_pipe(fifo_str, Some(5)).unwrap();
        assert_eq!(result, "hello from pipe");
        writer.join().unwrap();
    }

    // Cross-platform negative case tests

    #[test]
    fn test_read_nonexistent_pipe_errors() {
        let result = read_named_pipe("nonexistent_pipe_12345_xyz".to_string(), Some(0));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Failed to open") || err.contains("No such file"),
            "Expected open-failure error, got: {err}"
        );
    }

    #[test]
    fn test_read_pipe_zero_timeout_no_hang() {
        let start = ::std::time::Instant::now();
        let _ = read_named_pipe("nonexistent_timeout_test".to_string(), Some(0));
        assert!(
            start.elapsed().as_secs() < 2,
            "Zero-timeout should return immediately"
        );
    }

    #[test]
    fn test_read_pipe_default_timeout() {
        let start = ::std::time::Instant::now();
        let _ = read_named_pipe("nonexistent_default_timeout".to_string(), None);
        assert!(
            start.elapsed().as_secs() < 8,
            "Default timeout should be ~5s"
        );
    }

    // Linux tests

    #[cfg(unix)]
    #[test]
    fn test_read_fifo_basic() {
        use ::std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let fifo_path = dir.path().join("test_read_fifo");
        let fifo_str = fifo_path.to_str().unwrap().to_string();
        let c_path = ::std::ffi::CString::new(fifo_str.clone()).unwrap();
        unsafe { libc::mkfifo(c_path.as_ptr(), 0o644) };

        let fifo_clone = fifo_str.clone();
        let writer = ::std::thread::spawn(move || {
            let mut f = ::std::fs::OpenOptions::new()
                .write(true)
                .open(&fifo_clone)
                .unwrap();
            f.write_all(b"hello from pipe").unwrap();
        });

        let result = read_named_pipe(fifo_str, Some(5)).unwrap();
        assert_eq!(result, "hello from pipe");
        writer.join().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn test_read_fifo_large_data() {
        use ::std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let fifo_path = dir.path().join("test_large_fifo");
        let fifo_str = fifo_path.to_str().unwrap().to_string();
        let c_path = ::std::ffi::CString::new(fifo_str.clone()).unwrap();
        unsafe { libc::mkfifo(c_path.as_ptr(), 0o644) };

        let payload = "A".repeat(8192);
        let payload_clone = payload.clone();
        let fifo_clone = fifo_str.clone();
        let writer = ::std::thread::spawn(move || {
            let mut f = ::std::fs::OpenOptions::new()
                .write(true)
                .open(&fifo_clone)
                .unwrap();
            f.write_all(payload_clone.as_bytes()).unwrap();
        });

        let result = read_named_pipe(fifo_str, Some(5)).unwrap();
        assert_eq!(result.len(), 8192);
        writer.join().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn test_read_fifo_no_writer_timeout() {
        let dir = tempfile::tempdir().unwrap();
        let fifo_path = dir.path().join("test_empty_fifo");
        let fifo_str = fifo_path.to_str().unwrap().to_string();
        let c_path = ::std::ffi::CString::new(fifo_str.clone()).unwrap();
        unsafe { libc::mkfifo(c_path.as_ptr(), 0o644) };

        let start = ::std::time::Instant::now();
        let result = read_named_pipe(fifo_str, Some(1));
        assert!(result.is_err());
        assert!(start.elapsed().as_secs() <= 3, "Should timeout, not hang");
    }

    #[cfg(unix)]
    #[test]
    fn test_read_fifo_utf8() {
        use ::std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let fifo_path = dir.path().join("test_utf8_fifo");
        let fifo_str = fifo_path.to_str().unwrap().to_string();
        let c_path = ::std::ffi::CString::new(fifo_str.clone()).unwrap();
        unsafe { libc::mkfifo(c_path.as_ptr(), 0o644) };

        let fifo_clone = fifo_str.clone();
        let writer = ::std::thread::spawn(move || {
            let mut f = ::std::fs::OpenOptions::new()
                .write(true)
                .open(&fifo_clone)
                .unwrap();
            f.write_all("こんにちは".as_bytes()).unwrap();
        });

        let result = read_named_pipe(fifo_str, Some(5)).unwrap();
        assert_eq!(result, "こんにちは");
        writer.join().unwrap();
    }

    // Windows tests

    #[cfg(target_os = "windows")]
    #[test]
    fn test_read_windows_pipe_not_found() {
        let result = read_named_pipe("nonexistent_pipe_xyz_12345".to_string(), Some(0));
        assert!(result.is_err());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_read_windows_full_path_accepted() {
        let result = read_named_pipe(r"\\.\pipe\nonexistent_full_path_test".to_string(), Some(0));
        assert!(result.is_err()); // doesn't exist, but path not double-prefixed
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_read_windows_short_name_expanded() {
        let result = read_named_pipe("short_name_test_xyz".to_string(), Some(0));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains(r"\\.\pipe\short_name_test_xyz"),
            "Short name should expand: {err}"
        );
    }
}
