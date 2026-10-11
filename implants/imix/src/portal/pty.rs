use anyhow::Result;
use pb::portal::{BytesPayload, BytesPayloadKind, Mote, mote::Payload};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[cfg(not(target_os = "solaris"))]
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};

#[cfg(all(not(target_os = "windows"), not(target_os = "solaris")))]
use std::path::Path;

/// A single PTY session with its writer, process/master handle, and cancel channel.
struct PtySession {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    // Keep the master PTY handle alive for the lifetime of the session.
    // Dropping it closes the PTY fd, which would kill the shell process.
    #[cfg(not(target_os = "solaris"))]
    _master: Box<dyn MasterPty + Send>,
    #[cfg(target_os = "solaris")]
    _child_pid: libc::pid_t,
    _cancel_tx: mpsc::Sender<()>,
}

#[cfg(target_os = "solaris")]
impl Drop for PtySession {
    fn drop(&mut self) {
        unsafe {
            libc::kill(self._child_pid, libc::SIGHUP);
            let mut status = 0;
            libc::waitpid(self._child_pid, &mut status, libc::WNOHANG);
        }
    }
}

/// Manages PTY sessions keyed by stream_id.
pub struct PtyManager {
    sessions: HashMap<String, PtySession>,
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Process an incoming PTY mote. If no session exists for the stream_id,
    /// a new PTY is spawned. Input data is written to the PTY's stdin.
    pub async fn handle_mote(
        &mut self,
        stream_id: String,
        data: Vec<u8>,
        out_tx: mpsc::Sender<Mote>,
    ) -> Result<()> {
        if !self.sessions.contains_key(&stream_id) {
            // Spawn a new PTY session
            let session = spawn_pty_session(stream_id.clone(), out_tx.clone())?;
            self.sessions.insert(stream_id.clone(), session);
        }

        if let Some(session) = self.sessions.get(&stream_id) {
            // Write input data to the PTY (skip empty data which is just the init mote)
            if !data.is_empty()
                && let Ok(mut writer) = session.writer.lock()
            {
                let _ = writer.write_all(&data);
            }
        }

        Ok(())
    }
}

/// Spawn a new PTY process using portable-pty (Linux, macOS, Windows).
#[cfg(not(target_os = "solaris"))]
fn spawn_pty_session(stream_id: String, out_tx: mpsc::Sender<Mote>) -> Result<PtySession> {
    let pty_system = native_pty_system();

    let pair = pty_system.openpty(PtySize {
        rows: 48,
        cols: 160,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    // Determine the shell command
    let mut cmd_builder = {
        #[cfg(not(target_os = "windows"))]
        {
            if Path::new("/bin/bash").exists() {
                CommandBuilder::new("/bin/bash")
            } else {
                CommandBuilder::new("/bin/sh")
            }
        }
        #[cfg(target_os = "windows")]
        CommandBuilder::new("cmd.exe")
    };

    // Set TERM so that terminal-dependent commands (clear, reset, etc.) work
    cmd_builder.env("TERM", "xterm-256color");

    let child = pair.slave.spawn_command(cmd_builder)?;

    let reader = pair.master.try_clone_reader()?;
    let writer = pair.master.take_writer()?;
    let writer = Arc::new(Mutex::new(writer));

    let (cancel_tx, _cancel_rx) = mpsc::channel::<()>(1);

    // Use spawn_blocking for the PTY reader since portable-pty's read() is
    // synchronous blocking I/O that would starve the tokio async executor.
    let stream_id_clone = stream_id.clone();
    let out_tx_clone = out_tx.clone();
    tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Handle::current();
        let mut reader = reader;
        let mut child = child;
        let mut seq_id: u64 = 0;
        loop {
            let mut buffer = [0u8; 1024];
            let n = match reader.read(&mut buffer[..]) {
                Ok(n) if n > 0 => n,
                Ok(_) => {
                    // Check if process exited
                    if let Ok(Some(_status)) = child.try_wait() {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Err(_err) => {
                    #[cfg(feature = "print_debug")]
                    log::error!("PTY read error for {}: {}", stream_id_clone, _err);
                    break;
                }
            };

            seq_id += 1;
            let mote = Mote {
                stream_id: stream_id_clone.clone(),
                seq_id,
                payload: Some(Payload::Bytes(BytesPayload {
                    data: buffer[..n].to_vec(),
                    kind: BytesPayloadKind::Pty as i32,
                })),
            };

            if rt.block_on(out_tx_clone.send(mote)).is_err() {
                break;
            }
        }

        // Send close mote when PTY exits
        let close_mote = Mote {
            stream_id: stream_id_clone,
            seq_id: seq_id + 1,
            payload: Some(Payload::Bytes(BytesPayload {
                data: b"PTY session ended".to_vec(),
                kind: BytesPayloadKind::Close as i32,
            })),
        };
        let _ = rt.block_on(out_tx_clone.send(close_mote));
    });

    Ok(PtySession {
        writer,
        _master: pair.master,
        _cancel_tx: cancel_tx,
    })
}

/// Spawn a native POSIX + STREAMS PTY session on Solaris / illumos.
#[cfg(target_os = "solaris")]
fn spawn_pty_session(stream_id: String, out_tx: mpsc::Sender<Mote>) -> Result<PtySession> {
    use std::fs::File;
    use std::os::unix::io::FromRawFd;

    const PTEM: &[u8] = b"ptem\0";
    const LDTERM: &[u8] = b"ldterm\0";

    let (master_fd, slave_fd) = unsafe {
        let fdm = libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY);
        if fdm < 0 {
            return Err(anyhow::anyhow!(
                "Failed to open master PTY via posix_openpt"
            ));
        }
        if libc::grantpt(fdm) < 0 || libc::unlockpt(fdm) < 0 {
            libc::close(fdm);
            return Err(anyhow::anyhow!("Failed to grant/unlock slave PTY"));
        }
        let pts_name = libc::ptsname(fdm);
        if pts_name.is_null() {
            libc::close(fdm);
            return Err(anyhow::anyhow!("Failed to get slave PTY name"));
        }
        let fds = libc::open(pts_name, libc::O_RDWR | libc::O_NOCTTY);
        if fds < 0 {
            libc::close(fdm);
            return Err(anyhow::anyhow!("Failed to open slave PTY"));
        }

        // Push Solaris STREAMS terminal emulation and line discipline modules
        if libc::ioctl(fds, libc::I_PUSH, PTEM.as_ptr()) < 0
            || libc::ioctl(fds, libc::I_PUSH, LDTERM.as_ptr()) < 0
        {
            libc::close(fdm);
            libc::close(fds);
            return Err(anyhow::anyhow!(
                "Failed to push STREAMS modules to slave PTY"
            ));
        }

        let ws = libc::winsize {
            ws_row: 48,
            ws_col: 160,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let _ = libc::ioctl(fds, libc::TIOCSWINSZ, &ws);

        (fdm, fds)
    };

    let pid = unsafe { libc::fork() };
    if pid < 0 {
        unsafe {
            libc::close(master_fd);
            libc::close(slave_fd);
        }
        return Err(anyhow::anyhow!("Failed to fork PTY child process"));
    }

    if pid == 0 {
        // Child process
        unsafe {
            libc::close(master_fd);

            if libc::setsid() < 0
                || libc::ioctl(slave_fd, libc::TIOCSCTTY, 0) < 0
                || libc::dup2(slave_fd, 0) < 0
                || libc::dup2(slave_fd, 1) < 0
                || libc::dup2(slave_fd, 2) < 0
            {
                libc::_exit(1);
            }

            if slave_fd > 2 {
                libc::close(slave_fd);
            }

            // Set TERM environment variable
            let term = std::ffi::CString::new("TERM=xterm-256color").unwrap();
            libc::putenv(term.into_raw());

            let shell_path = if std::path::Path::new("/bin/bash").exists() {
                c"/bin/bash"
            } else {
                c"/bin/sh"
            };

            let args = [shell_path.as_ptr(), std::ptr::null()];
            libc::execv(shell_path.as_ptr(), args.as_ptr());
            libc::_exit(1);
        }
    }

    // Parent process
    unsafe {
        libc::close(slave_fd);
    }

    let master_file = unsafe { File::from_raw_fd(master_fd) };
    let writer_file = master_file.try_clone()?;
    let writer: Arc<Mutex<Box<dyn Write + Send>>> = Arc::new(Mutex::new(Box::new(writer_file)));

    let (cancel_tx, _cancel_rx) = mpsc::channel::<()>(1);
    let stream_id_clone = stream_id.clone();
    let out_tx_clone = out_tx.clone();

    tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Handle::current();
        let mut reader = master_file;
        let mut seq_id: u64 = 0;
        loop {
            let mut buffer = [0u8; 1024];
            let n = match reader.read(&mut buffer[..]) {
                Ok(n) if n > 0 => n,
                Ok(_) => {
                    let mut status = 0;
                    let wait_res = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
                    if wait_res != 0 {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Err(_err) => {
                    #[cfg(feature = "print_debug")]
                    log::error!("PTY read error for {}: {}", stream_id_clone, _err);
                    break;
                }
            };

            seq_id += 1;
            let mote = Mote {
                stream_id: stream_id_clone.clone(),
                seq_id,
                payload: Some(Payload::Bytes(BytesPayload {
                    data: buffer[..n].to_vec(),
                    kind: BytesPayloadKind::Pty as i32,
                })),
            };

            if rt.block_on(out_tx_clone.send(mote)).is_err() {
                break;
            }
        }

        // Send close mote when PTY exits
        let close_mote = Mote {
            stream_id: stream_id_clone,
            seq_id: seq_id + 1,
            payload: Some(Payload::Bytes(BytesPayload {
                data: b"PTY session ended".to_vec(),
                kind: BytesPayloadKind::Close as i32,
            })),
        };
        let _ = rt.block_on(out_tx_clone.send(close_mote));
    });

    Ok(PtySession {
        writer,
        _child_pid: pid,
        _cancel_tx: cancel_tx,
    })
}
