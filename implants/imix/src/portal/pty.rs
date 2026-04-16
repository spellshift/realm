use anyhow::Result;
use pb::portal::{BytesPayload, BytesPayloadKind, Mote, mote::Payload};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[cfg(not(target_os = "windows"))]
use std::path::Path;

/// A single PTY session with its writer and cancel channel.
struct PtySession {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    _cancel_tx: mpsc::Sender<()>,
}

/// Manages PTY sessions keyed by stream_id.
pub struct PtyManager {
    sessions: HashMap<String, PtySession>,
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
            if !data.is_empty() {
                if let Ok(mut writer) = session.writer.lock() {
                    let _ = writer.write_all(&data);
                }
            }
        }

        Ok(())
    }

    /// Remove a session by stream_id (e.g. on close).
    pub fn remove_session(&mut self, stream_id: &str) {
        self.sessions.remove(stream_id);
    }
}

/// Spawn a new PTY process and return a PtySession with a writer handle.
/// A background task reads PTY output and sends it as portal motes.
fn spawn_pty_session(stream_id: String, out_tx: mpsc::Sender<Mote>) -> Result<PtySession> {
    let pty_system = native_pty_system();

    let pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    // Determine the shell command
    let cmd_builder = {
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

    let mut child = pair.slave.spawn_command(cmd_builder)?;

    let mut reader = pair.master.try_clone_reader()?;
    let writer = pair.master.take_writer()?;
    let writer = Arc::new(Mutex::new(writer));

    let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);

    // Spawn task: read PTY output -> send as portal motes
    let stream_id_clone = stream_id.clone();
    let out_tx_clone = out_tx.clone();
    tokio::spawn(async move {
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
                    // Small sleep to avoid busy loop on zero reads
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    continue;
                }
                Err(_err) => {
                    #[cfg(debug_assertions)]
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

            if out_tx_clone.send(mote).await.is_err() {
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
        let _ = out_tx_clone.send(close_mote).await;
    });

    // Spawn a cancellation watcher that kills the child on cancel
    let stream_id_for_cancel = stream_id.clone();
    tokio::spawn(async move {
        let _ = cancel_rx.recv().await;
        #[cfg(debug_assertions)]
        log::info!("PTY session {} cancelled", stream_id_for_cancel);
    });

    Ok(PtySession {
        writer,
        _cancel_tx: cancel_tx,
    })
}
