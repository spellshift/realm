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
    use ::std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::ReadFile;
    use windows_sys::Win32::System::Pipes::PeekNamedPipe;

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
    let limit = max_bytes.filter(|&n| n > 0).map(|n| n as usize);
    let mut result: Vec<u8> = Vec::new();

    loop {
        // Check how many bytes are currently available without blocking.
        let mut bytes_avail: u32 = 0;
        let peek_ok = unsafe {
            PeekNamedPipe(
                handle,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                &mut bytes_avail,
                core::ptr::null_mut(),
            )
        };
        if peek_ok == 0 || bytes_avail == 0 {
            // Either the pipe errored (writer closed) or no data is queued.
            break;
        }

        let to_read = match limit {
            Some(max) => {
                let remaining = max.saturating_sub(result.len());
                if remaining == 0 {
                    break;
                }
                (bytes_avail as usize).min(remaining)
            }
            None => bytes_avail as usize,
        };

        let prev_len = result.len();
        result.resize(prev_len + to_read, 0u8);
        let mut bytes_read: u32 = 0;
        let read_ok = unsafe {
            ReadFile(
                handle,
                result[prev_len..].as_mut_ptr(),
                to_read as u32,
                &mut bytes_read,
                core::ptr::null_mut(),
            )
        };
        result.truncate(prev_len + bytes_read as usize);
        if read_ok == 0 {
            break;
        }
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

    /// Regression test: 3 × 1024-byte chunks written before a reader connects.
    ///
    /// On Windows, when a client connects to a named pipe the server's buffered
    /// writes (3 × 1024 = 3072 bytes) are flushed to the reader all at once.
    /// A subsequent PeekNamedPipe call returns 0 bytes available, so the read
    /// loop must stop without blocking. Any implementation that relies on
    /// read_to_end / read_exact would hang here because no EOF is ever sent.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_read_windows_three_chunks_no_eof_hang() {
        use windows_sys::Win32::Storage::FileSystem::{WriteFile, PIPE_ACCESS_OUTBOUND};
        use windows_sys::Win32::System::Pipes::{
            ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE, PIPE_WAIT,
        };

        // Unique pipe name for this test run.
        let pid = ::std::process::id();
        let pipe_name = format!(r"\\.\pipe\realm_test_3chunks_{}", pid);
        let wide: Vec<u16> = pipe_name.encode_utf16().chain(core::iter::once(0)).collect();

        // Create the server-side pipe handle (outbound / write side).
        // Cast to isize so the value is Send-able across threads (raw pointers are !Send
        // by default but Windows HANDLEs are safe to use from any thread).
        let server_raw: isize = unsafe {
            CreateNamedPipeW(
                wide.as_ptr(),
                PIPE_ACCESS_OUTBOUND,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                1,     // max instances
                16384, // out-buffer
                0,     // in-buffer
                0,     // default timeout
                core::ptr::null(),
            )
        } as isize;
        assert!(server_raw != -1, "CreateNamedPipeW failed");

        // Spawn a thread that waits for a client to connect then writes 3 × 1024 bytes.
        let server_thread = ::std::thread::spawn(move || {
            let server = server_raw as windows_sys::Win32::Foundation::HANDLE;
            unsafe { ConnectNamedPipe(server, core::ptr::null_mut()) };

            let chunk = vec![b'A'; 1024];
            for _ in 0..3 {
                let mut written: u32 = 0;
                unsafe {
                    WriteFile(
                        server,
                        chunk.as_ptr(),
                        chunk.len() as u32,
                        &mut written,
                        core::ptr::null_mut(),
                    )
                };
            }
            // Close the handle so the server side is gone — the reader must
            // not block waiting for more data after PeekNamedPipe returns 0.
            unsafe { windows_sys::Win32::Foundation::CloseHandle(server) };
        });

        // Give the server thread a moment to call ConnectNamedPipe before we open.
        ::std::thread::sleep(::std::time::Duration::from_millis(50));

        // Open the client side in a thread so we don't block the test harness.
        // Sleep briefly after opening so the server thread's WriteFile calls
        // complete before our PeekNamedPipe loop runs.
        let pipe_name_clone = pipe_name.clone();
        let reader_thread = ::std::thread::spawn(move || {
            ::std::thread::sleep(::std::time::Duration::from_millis(100));
            read_named_pipe(pipe_name_clone, None)
        });

        let result = reader_thread.join().unwrap();
        server_thread.join().unwrap();

        let data = result.expect("read_named_pipe must not hang or error");
        assert_eq!(
            data.len(),
            3072,
            "Expected 3 x 1024 = 3072 bytes, got {}",
            data.len()
        );
        assert!(data.chars().all(|c| c == 'A'), "All bytes should be 'A'");
    }
}
