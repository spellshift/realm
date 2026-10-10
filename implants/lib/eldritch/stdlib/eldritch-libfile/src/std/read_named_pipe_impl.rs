use alloc::format;
use alloc::string::String;
use alloc::vec;

/// Default read buffer size.
const DEFAULT_BUF_SIZE: usize = 4096;

/// Read data from a named pipe.
///
/// - Windows: opens `\\.\pipe\<name>`
/// - Unix: opens FIFO at `<name>`
/// - Other: returns error
///
/// If max_bytes is Some, reads up to that many bytes.
/// If None, reads all available data to EOF.
pub fn read_named_pipe(name: String, max_bytes: Option<i64>) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        read_named_pipe_windows(&name, max_bytes)
    }

    #[cfg(all(unix, not(target_os = "windows")))]
    {
        read_named_pipe_unix(&name, max_bytes)
    }

    #[cfg(not(any(target_os = "windows", unix)))]
    {
        let _ = (name, max_bytes);
        Err("read_named_pipe is only supported on Windows, Linux, macOS, and BSD".to_string())
    }
}

#[cfg(target_os = "windows")]
fn read_named_pipe_windows(name: &str, max_bytes: Option<i64>) -> Result<String, String> {
    use ::std::io::Read;
    use ::std::os::windows::io::AsRawHandle;

    // Build pipe path: \\.\pipe\<name>
    let pipe_path = if name.starts_with(r"\\") {
        name.to_string()
    } else {
        format!(r"\\.\pipe\{}", name)
    };

    let file = ::std::fs::OpenOptions::new()
        .read(true)
        .open(&pipe_path)
        .map_err(|e| format!("Failed to open pipe '{}': {}", pipe_path, e))?;

    let handle = file.as_raw_handle();

    // Poll with PeekNamedPipe - non-blocking check for available data. Retry briefly after connect: some pipe servers write in response
    // to a client connecting, so data may arrive shortly after open (don't want to miss the pipe drain if queued data exists in RAM)
    use windows_sys::Win32::System::Pipes::PeekNamedPipe;

    const MAX_RETRIES: u32 = 20; // dunno about the 20 peek loop TODO: figure out how to peek until 0 bytes?
    const RETRY_DELAY_MS: u64 = 100;
    const DRAIN_DELAY_MS: u64 = 10;

    let mut avail: u32 = 0;
    for _attempt in 0..MAX_RETRIES {
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
            let err = ::std::io::Error::last_os_error();
        }

        if avail > 0 {
            break;
        }
        ::std::thread::sleep(::std::time::Duration::from_millis(RETRY_DELAY_MS));
    }

    if avail == 0 {
        return Ok(String::new());
    }

    // Drain all available data. Peek+read in loop to catch multi-message writes
    // After each read, retry peek multiple times before giving up - server may
    // need time between writes (tried to make it as compatible as possible for variety of pipewriting tools)
    const DRAIN_RETRIES: u32 = 5;
    let max_read = max_bytes.map(|n| n as usize).unwrap_or(usize::MAX);
    let mut result = Vec::new();
    let mut buf = vec![0u8; DEFAULT_BUF_SIZE];
    let mut empty_peeks = 0u32;

    loop {
        let mut chunk_avail: u32 = 0;
        let peek_ok = unsafe {
            PeekNamedPipe(
                handle,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                &mut chunk_avail,
                core::ptr::null_mut(),
            )
        };
        if peek_ok == 0 {
            break; // pipe broken
        }

        if chunk_avail == 0 {
            empty_peeks += 1;
            if empty_peeks > DRAIN_RETRIES {
                break; // no more data after retries
            }
            ::std::thread::sleep(::std::time::Duration::from_millis(DRAIN_DELAY_MS));
            continue;
        }

        // Data available - reset empty counter, read
        empty_peeks = 0;

        let to_read = (chunk_avail as usize)
            .min(buf.len())
            .min(max_read - result.len());
        if to_read == 0 {
            break;
        }

        match (&file).read(&mut buf[..to_read]) {
            Ok(0) => break,
            Ok(n) => {
                result.extend_from_slice(&buf[..n]);
            }
            Err(_) => break,
        }

        if result.len() >= max_read {
            break;
        }
    }

    if result.is_empty() {
        return Ok(String::new());
    }

    String::from_utf8(result).map_err(|e| format!("Pipe data is not valid UTF-8: {}", e))
}

#[cfg(unix)]
fn read_named_pipe_unix(name: &str, max_bytes: Option<i64>) -> Result<String, String> {
    use ::std::io::Read;

    let mut file = ::std::fs::File::open(name)
        .map_err(|e| format!("Failed to open pipe '{}': {}", name, e))?;

    let mut result = Vec::new();

    match max_bytes {
        Some(n) if n > 0 => {
            let n = n as usize;
            let mut buf = vec![0u8; n.min(DEFAULT_BUF_SIZE)];
            let mut remaining = n;
            while remaining > 0 {
                let to_read = remaining.min(buf.len());
                match file.read(&mut buf[..to_read]) {
                    Ok(0) => break,
                    Ok(bytes_read) => {
                        result.extend_from_slice(&buf[..bytes_read]);
                        remaining -= bytes_read;
                    }
                    Err(_) => break,
                }
            }
        }
        _ => {
            file.read_to_end(&mut result)
                .map_err(|e| format!("Failed to read pipe '{}': {}", name, e))?;
        }
    }

    String::from_utf8(result).map_err(|e| format!("Pipe data is not valid UTF-8: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Cross-platform tests

    #[test]
    fn test_read_nonexistent_pipe_errors() {
        let result = read_named_pipe("nonexistent_pipe_12345_xyz".to_string(), None);
        assert!(result.is_err());
    }

    // Unix tests

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

        let result = read_named_pipe(fifo_str, None).unwrap();
        assert_eq!(result, "hello from pipe");
        writer.join().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn test_read_fifo_max_bytes() {
        use ::std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let fifo_path = dir.path().join("test_max_bytes_fifo");
        let fifo_str = fifo_path.to_str().unwrap().to_string();
        let c_path = ::std::ffi::CString::new(fifo_str.clone()).unwrap();
        unsafe { libc::mkfifo(c_path.as_ptr(), 0o644) };

        let fifo_clone = fifo_str.clone();
        let writer = ::std::thread::spawn(move || {
            let mut f = ::std::fs::OpenOptions::new()
                .write(true)
                .open(&fifo_clone)
                .unwrap();
            f.write_all(b"hello from pipe with extra data").unwrap();
        });

        // Read only 5 bytes
        let result = read_named_pipe(fifo_str, Some(5)).unwrap();
        assert_eq!(result, "hello");
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

        let result = read_named_pipe(fifo_str, None).unwrap();
        assert_eq!(result.len(), 8192);
        writer.join().unwrap();
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

        let result = read_named_pipe(fifo_str, None).unwrap();
        assert_eq!(result, "こんにちは");
        writer.join().unwrap();
    }

    // Windows tests

    #[cfg(target_os = "windows")]
    #[test]
    fn test_read_windows_pipe_not_found() {
        let result = read_named_pipe("nonexistent_pipe_xyz_12345".to_string(), None);
        assert!(result.is_err());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_read_windows_full_path_accepted() {
        let result = read_named_pipe(r"\\.\pipe\nonexistent_full_path_test".to_string(), None);
        assert!(result.is_err());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_read_windows_short_name_expanded() {
        let result = read_named_pipe("short_name_test_xyz".to_string(), None);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains(r"\\.\pipe\short_name_test_xyz"),
            "Short name should expand: {err}"
        );
    }
}
